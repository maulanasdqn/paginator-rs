use paginator_rs::{CursorDirection, CursorValue, PaginationParams, PaginatorError, PaginatorResponse, PaginatorResponseMeta};
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::{Connection, Surreal};

#[derive(Debug, serde::Deserialize)]
pub struct CountResult {
    pub count: i64,
}

pub async fn paginate_query<T, C>(
    db: &Surreal<C>,
    base_query: &str,
    params: &PaginationParams,
) -> Result<PaginatorResponse<T>, PaginatorError>
where
    T: DeserializeOwned + Serialize,
    C: Connection,
{
    let total = if params.disable_total_count {
        None
    } else {
        let mut count_query = if base_query.trim().to_uppercase().starts_with("SELECT") {
            let query_upper = base_query.to_uppercase();
            if let Some(from_pos) = query_upper.find("FROM") {
                let after_from = &base_query[from_pos..];
                format!("SELECT count() {}", after_from)
            } else {
                return Err(PaginatorError::Custom(
                    "Invalid query: missing FROM clause".to_string(),
                ));
            }
        } else {
            return Err(PaginatorError::Custom(
                "Query must start with SELECT".to_string(),
            ));
        };

        if let Some(where_clause) = params.to_surrealql_where() {
            let query_upper = count_query.to_uppercase();
            if query_upper.contains(" WHERE ") {
                count_query.push_str(&format!(" AND {}", where_clause));
            } else {
                count_query.push_str(&format!(" WHERE {}", where_clause));
            }
        }

        let count_result: Vec<CountResult> = db
            .query(&count_query)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?
            .take(0)
            .map_err(|e| PaginatorError::Custom(format!("Failed to extract count: {}", e)))?;

        count_result.first().map(|r| r.count as u32)
    };

    let mut paginated_query = base_query.to_string();

    if let Some(where_clause) = params.to_surrealql_where() {
        let query_upper = paginated_query.to_uppercase();
        if query_upper.contains(" WHERE ") {
            paginated_query.push_str(&format!(" AND {}", where_clause));
        } else {
            paginated_query.push_str(&format!(" WHERE {}", where_clause));
        }
    }

    if let Some(ref cursor) = params.cursor {
        let operator = match cursor.direction {
            CursorDirection::After => match params.sort_direction.as_ref() {
                Some(paginator_rs::SortDirection::Desc) => "<",
                _ => ">",
            },
            CursorDirection::Before => match params.sort_direction.as_ref() {
                Some(paginator_rs::SortDirection::Desc) => ">",
                _ => "<",
            },
        };

        let cursor_value = match &cursor.value {
            CursorValue::String(s) => format!("'{}'", s.replace('\'', "\\'")),
            CursorValue::Int(i) => i.to_string(),
            CursorValue::Float(f) => f.to_string(),
        };

        let query_upper = paginated_query.to_uppercase();
        if query_upper.contains(" WHERE ") {
            paginated_query.push_str(&format!(" AND {} {} {}", cursor.field, operator, cursor_value));
        } else {
            paginated_query.push_str(&format!(" WHERE {} {} {}", cursor.field, operator, cursor_value));
        }
    }

    if let Some(ref sort_field) = params.sort_by {
        let direction = match params.sort_direction.as_ref() {
            Some(paginator_rs::SortDirection::Desc) => "DESC",
            _ => "ASC",
        };
        paginated_query.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
    }

    if params.cursor.is_some() {
        paginated_query.push_str(&format!(" LIMIT {}", params.limit() + 1));
    } else {
        paginated_query.push_str(&format!(
            " LIMIT {} START {}",
            params.limit(),
            params.offset()
        ));
    }

    let mut data: Vec<T> = db
        .query(&paginated_query)
        .await
        .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?
        .take(0)
        .map_err(|e| PaginatorError::Custom(format!("Failed to extract results: {}", e)))?;

    let meta = if params.cursor.is_some() {
        let has_next = data.len() > params.per_page as usize;
        if has_next {
            data.truncate(params.per_page as usize);
        }
        PaginatorResponseMeta::new_with_cursors(
            params.page,
            params.per_page,
            total,
            has_next,
            None,
            None,
        )
    } else if let Some(count) = total {
        PaginatorResponseMeta::new(params.page, params.per_page, count)
    } else {
        let has_next = data.len() as u32 > params.per_page;
        PaginatorResponseMeta::new_without_total(params.page, params.per_page, has_next)
    };

    Ok(PaginatorResponse {
        data,
        meta,
    })
}
