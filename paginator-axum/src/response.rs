use axum::{
    http::{HeaderMap, HeaderValue, Response},
    response::IntoResponse,
    Json,
};
use paginator_rs::{PaginationParams, PaginatorResponse, PaginatorResponseMeta};
use serde::Serialize;

#[derive(Debug)]
pub struct PaginatedJson<T>(pub PaginatorResponse<T>);

impl<T> PaginatedJson<T>
where
    T: Serialize,
{
    pub fn new(data: Vec<T>, params: &PaginationParams, total: u32) -> Self {
        Self(PaginatorResponse {
            data,
            meta: PaginatorResponseMeta::new(params.page, params.per_page, total),
        })
    }

    pub fn from_response(response: PaginatorResponse<T>) -> Self {
        Self(response)
    }
}

impl<T> IntoResponse for PaginatedJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response<axum::body::Body> {
        let mut headers = HeaderMap::new();

        if let Some(total) = self.0.meta.total {
            headers.insert(
                "X-Total-Count",
                HeaderValue::from_str(&total.to_string()).unwrap(),
            );
        }
        if let Some(total_pages) = self.0.meta.total_pages {
            headers.insert(
                "X-Total-Pages",
                HeaderValue::from_str(&total_pages.to_string()).unwrap(),
            );
        }
        headers.insert(
            "X-Current-Page",
            HeaderValue::from_str(&self.0.meta.page.to_string()).unwrap(),
        );
        headers.insert(
            "X-Per-Page",
            HeaderValue::from_str(&self.0.meta.per_page.to_string()).unwrap(),
        );

        let json_response = Json(&self.0);

        (headers, json_response).into_response()
    }
}
