use crate::parser::parse_filter;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use paginator_rs::{Cursor, Filter, PaginationParams, SearchParams, SortDirection};
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
    /// An encoded cursor from a previous response's `next_cursor`/`prev_cursor`.
    pub cursor: Option<String>,
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
        let mut cursor: Option<Cursor> = None;

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
                "cursor" => {
                    cursor = Some(Cursor::decode(&value).map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            format!("Invalid query params: cursor: {}", e),
                        )
                    })?)
                }
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
            cursor,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use paginator_rs::{CursorDirection, CursorValue};

    async fn extract(query: &str) -> Result<PaginationParams, (StatusCode, String)> {
        let request = Request::builder()
            .uri(format!("/users?{query}"))
            .body(())
            .unwrap();
        let (mut parts, _) = request.into_parts();
        PaginationQuery::from_request_parts(&mut parts, &())
            .await
            .map(|q| q.0)
    }

    #[tokio::test]
    async fn parses_cursor_param() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(42), CursorDirection::After);
        let encoded = cursor.encode().unwrap();
        let params = extract(&format!(
            "cursor={encoded}&per_page=5&sort_by=id&sort_direction=desc"
        ))
        .await
        .unwrap();
        assert_eq!(params.cursor, Some(cursor));
        assert_eq!(params.per_page, 5);
        assert_eq!(params.sort_by.as_deref(), Some("id"));
        assert_eq!(params.sort_direction, Some(SortDirection::Desc));
    }

    #[tokio::test]
    async fn rejects_invalid_cursor() {
        let (status, message) = extract("cursor=not-a-cursor").await.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(message.contains("cursor"), "{message}");
    }

    #[tokio::test]
    async fn no_cursor_by_default() {
        let params = extract("page=2&filter=age:gt:18").await.unwrap();
        assert!(params.cursor.is_none());
        assert_eq!(params.page, 2);
        assert_eq!(params.filters.len(), 1);
    }
}
