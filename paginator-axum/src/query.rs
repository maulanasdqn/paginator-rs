use crate::parser::parse_filter;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use paginator_rs::{Filter, PaginationParams, SearchParams, SortDirection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PaginationQuery(pub PaginationParams);

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationQueryParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_direction: Option<String>,
    #[serde(default)]
    pub filter: Vec<String>,
    pub search: Option<String>,
    pub search_fields: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl<S> FromRequestParts<S> for PaginationQuery
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Parse the query string directly: serde_urlencoded (used by axum's
        // Query extractor) cannot collect repeated `filter=` keys into a Vec.
        let query_str = parts.uri.query().unwrap_or("");

        let mut page = 1u32;
        let mut per_page = 20u32;
        let mut sort_by: Option<String> = None;
        let mut sort_direction: Option<SortDirection> = None;
        let mut filters: Vec<Filter> = Vec::new();
        let mut search_query: Option<String> = None;
        let mut search_fields: Option<String> = None;

        for (key, value) in form_urlencoded::parse(query_str.as_bytes()) {
            match key.as_ref() {
                "page" => {
                    page = value.parse::<u32>().map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            format!("Invalid query params: page: {}", value),
                        )
                    })?
                }
                "per_page" => {
                    per_page = value.parse::<u32>().map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            format!("Invalid query params: per_page: {}", value),
                        )
                    })?
                }
                "sort_by" => sort_by = Some(value.into_owned()),
                "sort_direction" => {
                    sort_direction = match value.to_lowercase().as_str() {
                        "asc" => Some(SortDirection::Asc),
                        "desc" => Some(SortDirection::Desc),
                        _ => None,
                    }
                }
                "filter" => {
                    if let Some(f) = parse_filter(&value) {
                        filters.push(f);
                    }
                }
                "search" => search_query = Some(value.into_owned()),
                "search_fields" => search_fields = Some(value.into_owned()),
                _ => {}
            }
        }

        let search = search_query.and_then(|query| {
            let fields: Vec<String> = search_fields
                .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            if fields.is_empty() {
                None
            } else {
                Some(SearchParams {
                    query,
                    fields,
                    case_sensitive: false,
                    exact_match: false,
                })
            }
        });

        Ok(PaginationQuery(PaginationParams {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
            sort_by,
            sort_direction,
            filters,
            search,
            disable_total_count: false,
            cursor: None,
        }))
    }
}
