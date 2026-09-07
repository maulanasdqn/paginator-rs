use paginator_rs::{
    Cursor, PaginationParams, PaginatorResponse, PaginatorResponseMeta, SortDirection,
};
use rocket::{
    http::{Header, Status},
    request::{self, FromRequest, Request},
    response::{self, Responder},
    serde::json::Json,
};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Pagination {
    pub params: PaginationParams,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Pagination {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let query = req.uri().query();

        let mut page = 1u32;
        let mut per_page = 20u32;
        let mut sort_by: Option<String> = None;
        let mut sort_direction: Option<SortDirection> = None;
        let mut cursor: Option<Cursor> = None;

        if let Some(query) = query {
            for (key, value) in query.segments() {
                match key {
                    "page" => {
                        if let Ok(p) = value.parse::<u32>() {
                            page = p.max(1);
                        }
                    }
                    "per_page" => {
                        if let Ok(pp) = value.parse::<u32>() {
                            per_page = pp.clamp(1, 100);
                        }
                    }
                    "sort_by" => {
                        sort_by = Some(value.to_string());
                    }
                    "sort_direction" => {
                        sort_direction = match value.to_lowercase().as_str() {
                            "asc" => Some(SortDirection::Asc),
                            "desc" => Some(SortDirection::Desc),
                            _ => None,
                        };
                    }
                    "cursor" => match Cursor::decode(value) {
                        Ok(decoded) => cursor = Some(decoded),
                        Err(_) => {
                            return request::Outcome::Error((Status::BadRequest, "invalid cursor"))
                        }
                    },
                    _ => {}
                }
            }
        }

        request::Outcome::Success(Pagination {
            params: PaginationParams {
                page,
                per_page,
                sort_by,
                sort_direction,
                filters: Vec::new(),
                search: None,
                disable_total_count: false,
                cursor,
            },
        })
    }
}

#[derive(Debug)]
pub struct PaginatedJson<T> {
    response: PaginatorResponse<T>,
}

impl<T> PaginatedJson<T>
where
    T: Serialize,
{
    pub fn new(data: Vec<T>, params: &PaginationParams, total: u32) -> Self {
        Self {
            response: PaginatorResponse {
                data,
                meta: PaginatorResponseMeta::new(params.page, params.per_page, total),
            },
        }
    }

    pub fn from_response(response: PaginatorResponse<T>) -> Self {
        Self { response }
    }
}

impl<'r, T> Responder<'r, 'static> for PaginatedJson<T>
where
    T: Serialize,
{
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let json = Json(&self.response);
        let mut response = json.respond_to(req)?;

        if let Some(total) = self.response.meta.total {
            response.set_header(Header::new("X-Total-Count", total.to_string()));
        }
        if let Some(total_pages) = self.response.meta.total_pages {
            response.set_header(Header::new("X-Total-Pages", total_pages.to_string()));
        }
        response.set_header(Header::new(
            "X-Current-Page",
            self.response.meta.page.to_string(),
        ));
        response.set_header(Header::new(
            "X-Per-Page",
            self.response.meta.per_page.to_string(),
        ));

        Ok(response)
    }
}

pub fn create_paginated_response<T>(
    data: Vec<T>,
    params: &PaginationParams,
    total: u32,
) -> PaginatedJson<T>
where
    T: Serialize,
{
    PaginatedJson::new(data, params, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use paginator_rs::{CursorDirection, CursorValue};
    use rocket::local::blocking::Client;
    use rocket::{get, routes};

    #[get("/users")]
    fn users(pagination: Pagination) -> String {
        serde_json::to_string(&pagination.params).unwrap()
    }

    fn client() -> Client {
        Client::tracked(rocket::build().mount("/", routes![users])).unwrap()
    }

    #[test]
    fn parses_cursor_param() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(42), CursorDirection::After);
        let encoded = cursor.encode().unwrap();
        let client = client();
        let response = client
            .get(format!("/users?cursor={encoded}&per_page=5"))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let params: PaginationParams =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();
        assert_eq!(params.cursor, Some(cursor));
        assert_eq!(params.per_page, 5);
    }

    #[test]
    fn rejects_invalid_cursor() {
        let client = client();
        let response = client.get("/users?cursor=not-a-cursor").dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn no_cursor_by_default() {
        let client = client();
        let response = client.get("/users?page=2").dispatch();
        let params: PaginationParams =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();
        assert!(params.cursor.is_none());
        assert_eq!(params.page, 2);
    }
}
