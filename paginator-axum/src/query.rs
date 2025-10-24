use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
};
use paginator_rs::{Filter, PaginationParams, SearchParams, SortDirection};
use serde::Deserialize;
use crate::parser::parse_filter;

#[derive(Debug, Clone)]
pub struct PaginationQuery(pub PaginationParams);

#[derive(Debug, Deserialize)]
pub struct PaginationQueryParams {
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

#[async_trait]
impl<S> FromRequestParts<S> for PaginationQuery
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params): Query<PaginationQueryParams> =
            Query::from_request_parts(parts, state).await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid query params: {}", e),
                )
            })?;

        let sort_direction = params
            .sort_direction
            .and_then(|s| match s.to_lowercase().as_str() {
                "asc" => Some(SortDirection::Asc),
                "desc" => Some(SortDirection::Desc),
                _ => None,
            });

        let filters: Vec<Filter> = params
            .filter
            .iter()
            .filter_map(|f| parse_filter(f))
            .collect();

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
            disable_total_count: false,
            cursor: None,
        }))
    }
}
