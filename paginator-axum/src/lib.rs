//! Axum web framework integration for paginator-rs
//!
//! This crate provides extractors and response types for seamless pagination in Axum applications.
//!
//! # Example
//!
//! ```ignore
//! use axum::{Router, routing::get, Json};
//! use paginator_axum::{PaginationQuery, PaginatedJson};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//! }
//!
//! async fn get_users(
//!     PaginationQuery(params): PaginationQuery,
//! ) -> PaginatedJson<User> {
//!     // Fetch users from database with params
//!     let users = vec![/* ... */];
//!
//!     // Create response (automatically serializes to JSON)
//!     PaginatedJson::new(users, &params, 100)
//! }
//!
//! let app = Router::new().route("/users", get(get_users));
//! ```

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{request::Parts, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use paginator_rs::{
    Filter, FilterOperator, FilterValue, PaginationParams, PaginatorResponse,
    PaginatorResponseMeta, SearchParams, SortDirection,
};
use serde::{Deserialize, Serialize};

/// Query parameter extractor for pagination
///
/// # Example
///
/// ```ignore
/// async fn handler(PaginationQuery(params): PaginationQuery) {
///     // params is PaginationParams
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PaginationQuery(pub PaginationParams);

#[derive(Debug, Deserialize)]
struct PaginationQueryParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_per_page")]
    per_page: u32,
    sort_by: Option<String>,
    #[serde(default)]
    sort_direction: Option<String>,
    #[serde(default)]
    filter: Vec<String>,
    search: Option<String>,
    search_fields: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

/// Parse filter string in format "field:operator:value"
fn parse_filter(filter_str: &str) -> Option<Filter> {
    let parts: Vec<&str> = filter_str.splitn(3, ':').collect();
    if parts.len() < 3 {
        return None;
    }

    let field = parts[0].to_string();
    let operator = match parts[1] {
        "eq" => FilterOperator::Eq,
        "ne" => FilterOperator::Ne,
        "gt" => FilterOperator::Gt,
        "lt" => FilterOperator::Lt,
        "gte" => FilterOperator::Gte,
        "lte" => FilterOperator::Lte,
        "like" => FilterOperator::Like,
        "ilike" => FilterOperator::ILike,
        "in" => FilterOperator::In,
        "not_in" => FilterOperator::NotIn,
        "is_null" => FilterOperator::IsNull,
        "is_not_null" => FilterOperator::IsNotNull,
        "between" => FilterOperator::Between,
        "contains" => FilterOperator::Contains,
        _ => return None,
    };

    let value_str = parts[2];

    // Parse value based on operator
    let value = match operator {
        FilterOperator::IsNull | FilterOperator::IsNotNull => FilterValue::Null,
        FilterOperator::In | FilterOperator::NotIn => {
            // Parse comma-separated values
            let values: Vec<FilterValue> = value_str
                .split(',')
                .filter_map(|v| {
                    let trimmed = v.trim();
                    // Try parsing as different types
                    if let Ok(i) = trimmed.parse::<i64>() {
                        Some(FilterValue::Int(i))
                    } else if let Ok(f) = trimmed.parse::<f64>() {
                        Some(FilterValue::Float(f))
                    } else if trimmed == "true" || trimmed == "false" {
                        Some(FilterValue::Bool(trimmed == "true"))
                    } else {
                        Some(FilterValue::String(trimmed.to_string()))
                    }
                })
                .collect();
            FilterValue::Array(values)
        }
        FilterOperator::Between => {
            // Parse range "min,max"
            let values: Vec<FilterValue> = value_str
                .split(',')
                .filter_map(|v| {
                    let trimmed = v.trim();
                    if let Ok(i) = trimmed.parse::<i64>() {
                        Some(FilterValue::Int(i))
                    } else if let Ok(f) = trimmed.parse::<f64>() {
                        Some(FilterValue::Float(f))
                    } else {
                        Some(FilterValue::String(trimmed.to_string()))
                    }
                })
                .collect();
            FilterValue::Array(values)
        }
        _ => {
            // Try parsing as different types
            if let Ok(i) = value_str.parse::<i64>() {
                FilterValue::Int(i)
            } else if let Ok(f) = value_str.parse::<f64>() {
                FilterValue::Float(f)
            } else if value_str == "true" || value_str == "false" {
                FilterValue::Bool(value_str == "true")
            } else {
                FilterValue::String(value_str.to_string())
            }
        }
    };

    Some(Filter {
        field,
        operator,
        value,
    })
}

#[async_trait]
impl<S> FromRequestParts<S> for PaginationQuery
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params): Query<PaginationQueryParams> =
            Query::from_request_parts(parts, state)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid query params: {}", e)))?;

        let sort_direction = params.sort_direction.and_then(|s| match s.to_lowercase().as_str() {
            "asc" => Some(SortDirection::Asc),
            "desc" => Some(SortDirection::Desc),
            _ => None,
        });

        // Parse filters
        let filters: Vec<Filter> = params
            .filter
            .iter()
            .filter_map(|f| parse_filter(f))
            .collect();

        // Parse search parameters
        let search = if let Some(query) = params.search {
            let fields: Vec<String> = params
                .search_fields
                .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            if !fields.is_empty() {
                Some(SearchParams {
                    query,
                    fields,
                    case_sensitive: false,
                    exact_match: false,
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(PaginationQuery(PaginationParams {
            page: params.page.max(1),
            per_page: params.per_page.max(1).min(100),
            sort_by: params.sort_by,
            sort_direction,
            filters,
            search,
        }))
    }
}

/// Paginated JSON response with HTTP headers
///
/// Automatically adds:
/// - X-Total-Count: Total number of items
/// - X-Total-Pages: Total number of pages
/// - X-Current-Page: Current page number
/// - X-Per-Page: Items per page
#[derive(Debug)]
pub struct PaginatedJson<T>(pub PaginatorResponse<T>);

impl<T> PaginatedJson<T>
where
    T: Serialize,
{
    /// Create a new paginated response
    pub fn new(data: Vec<T>, params: &PaginationParams, total: u32) -> Self {
        Self(PaginatorResponse {
            data,
            meta: PaginatorResponseMeta::new(params.page, params.per_page, total),
        })
    }

    /// Create from an existing PaginatorResponse
    pub fn from_response(response: PaginatorResponse<T>) -> Self {
        Self(response)
    }
}

impl<T> IntoResponse for PaginatedJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();

        // Add pagination metadata to headers
        headers.insert(
            "X-Total-Count",
            HeaderValue::from_str(&self.0.meta.total.to_string()).unwrap(),
        );
        headers.insert(
            "X-Total-Pages",
            HeaderValue::from_str(&self.0.meta.total_pages.to_string()).unwrap(),
        );
        headers.insert(
            "X-Current-Page",
            HeaderValue::from_str(&self.0.meta.page.to_string()).unwrap(),
        );
        headers.insert(
            "X-Per-Page",
            HeaderValue::from_str(&self.0.meta.per_page.to_string()).unwrap(),
        );

        // Create JSON response
        let json_response = Json(&self.0);

        (headers, json_response).into_response()
    }
}


/// Helper to create Link header for pagination (RFC 8288)
///
/// # Example
///
/// ```ignore
/// let link_header = create_link_header("/api/users", &params, &meta);
/// ```
pub fn create_link_header(base_url: &str, params: &PaginationParams, meta: &PaginatorResponseMeta) -> String {
    let mut links = Vec::new();

    // First page
    links.push(format!(
        "<{}?page=1&per_page={}>; rel=\"first\"",
        base_url, params.per_page
    ));

    // Previous page
    if meta.has_prev {
        links.push(format!(
            "<{}?page={}&per_page={}>; rel=\"prev\"",
            base_url,
            params.page - 1,
            params.per_page
        ));
    }

    // Next page
    if meta.has_next {
        links.push(format!(
            "<{}?page={}&per_page={}>; rel=\"next\"",
            base_url,
            params.page + 1,
            params.per_page
        ));
    }

    // Last page
    links.push(format!(
        "<{}?page={}&per_page={}>; rel=\"last\"",
        base_url, meta.total_pages, params.per_page
    ));

    links.join(", ")
}
