use paginator_rs::{
    KeysetPlan, PaginationParams, PaginatorError, PaginatorResponse, PaginatorResponseMeta,
    SortDirection,
};
use serde::Serialize;
use sqlx::{Database, Executor, FromRow};
use std::marker::PhantomData;

/// Validates that a field name is safe for use in SQL queries.
/// Only allows alphanumeric characters, underscores, and dots (for qualified names).
/// Returns an error if the field name contains potentially dangerous characters.
pub fn validate_field_name(field: &str) -> Result<(), PaginatorError> {
    if field.is_empty() {
        return Err(PaginatorError::Custom(
            "Field name cannot be empty".to_string(),
        ));
    }

    for c in field.chars() {
        if !c.is_alphanumeric() && c != '_' && c != '.' {
            return Err(PaginatorError::Custom(format!(
                "Invalid field name '{}': contains unsafe character '{}'",
                field, c
            )));
        }
    }

    Ok(())
}

/// Strip surrounding whitespace and a trailing semicolon so the query can be
/// used as a subquery.
pub(crate) fn clean_base_query(base_query: &str) -> &str {
    base_query.trim().trim_end_matches(';').trim_end()
}

/// `SELECT COUNT(*)` over the base query. With `wrap`, the base query becomes a
/// derived table ending in `WHERE 1=1` so filter and search predicates can be
/// appended with `AND`. Common table expressions are valid inside a derived
/// table on PostgreSQL, MySQL 8, and SQLite, so no special casing is needed.
pub(crate) fn count_sql(base_query: &str, wrap: bool) -> String {
    if wrap {
        format!("SELECT COUNT(*) FROM ({}) AS _base WHERE 1=1", base_query)
    } else {
        format!("SELECT COUNT(*) FROM ({}) AS count_subquery", base_query)
    }
}

/// The data query. With `wrap`, the base query becomes a derived table ending in
/// `WHERE 1=1` so filters, search, and the keyset predicate can be appended.
pub(crate) fn data_sql(base_query: &str, wrap: bool) -> String {
    if wrap {
        format!("SELECT * FROM ({}) AS _base WHERE 1=1", base_query)
    } else {
        base_query.to_string()
    }
}

/// The `ORDER BY` to apply. A cursor always orders by its own field, in query
/// order (reversed for `Before` cursors); otherwise `sort_by` is used as given.
pub(crate) fn order_for<'p>(
    params: &'p PaginationParams,
    plan: Option<&KeysetPlan<'p>>,
) -> Result<Option<(&'p str, SortDirection)>, PaginatorError> {
    if let Some(plan) = plan {
        validate_field_name(plan.field())?;
        return Ok(Some((plan.field(), plan.query_sort)));
    }
    if let Some(field) = params.sort_by.as_deref() {
        validate_field_name(field)?;
        return Ok(Some((
            field,
            params.sort_direction.unwrap_or(SortDirection::Asc),
        )));
    }
    Ok(None)
}

/// `LIMIT`/`OFFSET` for the data query. One extra row is fetched whenever
/// `has_next` cannot be derived from a total: in cursor mode and when the count
/// query is disabled. With a cursor the offset skips whole pages relative to the
/// cursor (relative cursor pagination); without one it is the usual page offset.
pub(crate) fn limit_offset(params: &PaginationParams, keyset: bool) -> (i64, i64) {
    let probe = keyset || params.disable_total_count;
    let limit = params.limit() as i64 + i64::from(probe);
    (limit, params.offset() as i64)
}

/// Turn fetched rows into a response, trimming the probe row where one was
/// fetched and deriving cursors in cursor mode.
pub(crate) fn finish<T: Serialize>(
    mut data: Vec<T>,
    params: &PaginationParams,
    plan: Option<&KeysetPlan<'_>>,
    total: Option<i64>,
) -> PaginatorResponse<T> {
    let total = total.map(|t| t as u32);
    let meta = if let Some(plan) = plan {
        PaginatorResponseMeta::from_cursor_page(&mut data, params, plan, total)
    } else if let Some(count) = total {
        PaginatorResponseMeta::new(params.page, params.per_page, count)
    } else {
        let has_next = data.len() > params.per_page as usize;
        data.truncate(params.per_page as usize);
        PaginatorResponseMeta::new_without_total(params.page, params.per_page, has_next)
    };
    PaginatorResponse { data, meta }
}

pub trait PaginateQuery<'q, DB: Database, T>
where
    T: Send + Unpin,
{
    fn paginate(self, params: &PaginationParams) -> PaginatedQuery<'q, DB, T>;
}

pub struct PaginatedQuery<'q, DB: Database, T> {
    query: &'q str,
    _phantom: PhantomData<(DB, T)>,
}

impl<'q, DB: Database, T> PaginatedQuery<'q, DB, T> {
    pub fn new(query: &'q str, _params: &PaginationParams) -> Self {
        Self {
            query,
            _phantom: PhantomData,
        }
    }

    pub async fn fetch<'e, E>(self, _executor: E) -> Result<PaginatorResponse<T>, PaginatorError>
    where
        E: Executor<'e, Database = DB>,
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin + Serialize,
    {
        let _count_query = format!("SELECT COUNT(*) FROM ({})", self.query);

        Err(PaginatorError::Custom(
            "SQLx integration requires database-specific query building. \
             See examples for proper implementation."
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_semicolon() {
        assert_eq!(clean_base_query("  SELECT 1;  "), "SELECT 1");
        assert_eq!(clean_base_query("SELECT 1"), "SELECT 1");
    }

    #[test]
    fn wraps_only_when_asked() {
        assert_eq!(data_sql("SELECT * FROM t", false), "SELECT * FROM t");
        assert_eq!(
            data_sql("SELECT * FROM t", true),
            "SELECT * FROM (SELECT * FROM t) AS _base WHERE 1=1"
        );
        assert_eq!(
            count_sql("WITH a AS (SELECT 1) SELECT * FROM a", true),
            "SELECT COUNT(*) FROM (WITH a AS (SELECT 1) SELECT * FROM a) AS _base WHERE 1=1"
        );
    }

    #[test]
    fn probes_one_extra_row_in_cursor_mode_and_without_count() {
        let mut params = PaginationParams::new(3, 10);
        assert_eq!(limit_offset(&params, false), (10, 20));
        assert_eq!(limit_offset(&params, true), (11, 20));
        params.disable_total_count = true;
        assert_eq!(limit_offset(&params, false), (11, 20));
    }
}
