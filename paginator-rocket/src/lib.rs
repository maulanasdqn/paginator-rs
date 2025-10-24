use paginator_rs::{PaginationParams, PaginatorResponse, PaginatorResponseMeta, SortDirection};
use rocket::{
    http::Header,
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
                            per_page = pp.max(1).min(100);
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
                cursor: None,
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
            response.set_header(Header::new(
                "X-Total-Count",
                total.to_string(),
            ));
        }
        if let Some(total_pages) = self.response.meta.total_pages {
            response.set_header(Header::new(
                "X-Total-Pages",
                total_pages.to_string(),
            ));
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
