use paginator_rs::{PaginationParams, PaginatorError, PaginatorResponse};
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
