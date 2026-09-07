use crate::validate_field_name;
use paginator_rs::{
    CursorValue, PaginationParams, PaginatorError, PaginatorResponse, PaginatorResponseMeta,
    SortDirection,
};
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::{Connection, Surreal};

/// Safely escapes a string value for use in SurrealQL queries.
/// This replaces single quotes and backslashes to prevent injection.
fn escape_string_value(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Append `condition` with `WHERE` or `AND` depending on whether the query
/// already has a `WHERE` clause.
fn append_condition(query: &mut String, condition: &str) {
    if query.to_uppercase().contains(" WHERE ") {
        query.push_str(&format!(" AND {}", condition));
    } else {
        query.push_str(&format!(" WHERE {}", condition));
    }
}

fn direction_keyword(direction: SortDirection) -> &'static str {
    match direction {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CountResult {
    pub count: i64,
}

/// Paginate `base_query`, applying the filters, search, sort, and cursor in `params`.
///
/// With a cursor, rows are ordered by the cursor field (`sort_direction`,
/// ascending by default), `per_page + 1` rows are fetched to detect the next
/// page, and `next_cursor`/`prev_cursor` are derived from the boundary rows.
/// `page` then acts as an offset in pages relative to the cursor.
pub async fn paginate_query<T, C>(
    db: &Surreal<C>,
    base_query: &str,
    params: &PaginationParams,
) -> Result<PaginatorResponse<T>, PaginatorError>
where
    T: DeserializeOwned + Serialize,
    C: Connection,
{
    let plan = params.keyset_plan().map_err(PaginatorError::Custom)?;

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
            append_condition(&mut count_query, &where_clause);
        }
        // Without GROUP ALL, `count()` yields one row per record.
        count_query.push_str(" GROUP ALL");

        let count_result: Vec<CountResult> = db
            .query(&count_query)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?
            .take(0)
            .map_err(|e| PaginatorError::Custom(format!("Failed to extract count: {}", e)))?;

        // An empty result set produces no group at all.
        Some(count_result.first().map(|r| r.count as u32).unwrap_or(0))
    };

    let mut paginated_query = base_query.to_string();

    if let Some(where_clause) = params.to_surrealql_where() {
        append_condition(&mut paginated_query, &where_clause);
    }

    if let Some(ref plan) = plan {
        // Validate cursor field name to prevent injection
        validate_field_name(plan.field())?;

        let cursor_value = match plan.value() {
            CursorValue::String(s) => format!("'{}'", escape_string_value(s)),
            CursorValue::Int(i) => i.to_string(),
            CursorValue::Float(f) => f.to_string(),
            CursorValue::Uuid(u) => format!("<uuid> '{}'", escape_string_value(u)),
        };

        append_condition(
            &mut paginated_query,
            &format!("{} {} {}", plan.field(), plan.operator(), cursor_value),
        );
        paginated_query.push_str(&format!(
            " ORDER BY {} {}",
            plan.field(),
            direction_keyword(plan.query_sort)
        ));
    } else if let Some(ref sort_field) = params.sort_by {
        // Validate sort field name to prevent injection
        validate_field_name(sort_field)?;

        paginated_query.push_str(&format!(
            " ORDER BY {} {}",
            sort_field,
            direction_keyword(params.sort_direction.unwrap_or(SortDirection::Asc))
        ));
    }

    // One extra row detects the next page when there is no total to derive it
    // from: in cursor mode and when the count query is disabled.
    let probe = plan.is_some() || params.disable_total_count;
    let limit = params.limit() + u32::from(probe);
    paginated_query.push_str(&format!(" LIMIT {} START {}", limit, params.offset()));

    let mut data: Vec<T> = db
        .query(&paginated_query)
        .await
        .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?
        .take(0)
        .map_err(|e| PaginatorError::Custom(format!("Failed to extract results: {}", e)))?;

    let meta = if let Some(ref plan) = plan {
        PaginatorResponseMeta::from_cursor_page(&mut data, params, plan, total)
    } else if let Some(count) = total {
        PaginatorResponseMeta::new(params.page, params.per_page, count)
    } else {
        let has_next = data.len() > params.per_page as usize;
        data.truncate(params.per_page as usize);
        PaginatorResponseMeta::new_without_total(params.page, params.per_page, has_next)
    };

    Ok(PaginatorResponse { data, meta })
}
