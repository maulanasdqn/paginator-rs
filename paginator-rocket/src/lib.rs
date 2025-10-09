//! Rocket web framework integration for paginator-rs
//!
//! This crate provides request guards and responders for seamless pagination in Rocket applications.
//!
//! # Example
//!
//! ```ignore
//! use rocket::{get, routes};
//! use paginator_rocket::{Pagination, PaginatedJson};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//! }
//!
//! #[get("/users")]
//! async fn get_users(pagination: Pagination) -> PaginatedJson<User> {
//!     // Fetch users from database with pagination.params
//!     let users = vec![/* ... */];
//!
//!     PaginatedJson::new(users, &pagination.params, 100)
//! }
//!
//! #[launch]
//! fn rocket() -> _ {
//!     rocket::build().mount("/api", routes![get_users])
//! }
//! ```

use paginator_rs::{PaginationParams, PaginatorResponse, PaginatorResponseMeta, SortDirection};
use rocket::{
    http::Header,
    request::{self, FromRequest, Request},
    response::{self, Responder},
    serde::json::Json,
};
use serde::Serialize;

/// Request guard for pagination parameters
///
/// # Example
///
/// ```ignore
/// #[get("/items")]
/// fn items(pagination: Pagination) {
///     let params = pagination.params;
///     // Use params...
/// }
/// ```
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
            },
        })
    }
}

/// Paginated JSON responder
///
/// Automatically adds pagination metadata to response headers
#[derive(Debug)]
pub struct PaginatedJson<T> {
    response: PaginatorResponse<T>,
}

impl<T> PaginatedJson<T>
where
    T: Serialize,
{
    /// Create a new paginated response
    pub fn new(data: Vec<T>, params: &PaginationParams, total: u32) -> Self {
        Self {
            response: PaginatorResponse {
                data,
                meta: PaginatorResponseMeta::new(params.page, params.per_page, total),
            },
        }
    }

    /// Create from an existing PaginatorResponse
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

        // Add pagination headers
        response.set_header(Header::new(
            "X-Total-Count",
            self.response.meta.total.to_string(),
        ));
        response.set_header(Header::new(
            "X-Total-Pages",
            self.response.meta.total_pages.to_string(),
        ));
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

/// Helper to create a paginated response from query results
///
/// # Example
///
/// ```ignore
/// #[get("/users")]
/// fn users(pagination: Pagination) -> PaginatedJson<User> {
///     let users = fetch_users(&pagination.params);
///     create_paginated_response(users, &pagination.params, 100)
/// }
/// ```
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
