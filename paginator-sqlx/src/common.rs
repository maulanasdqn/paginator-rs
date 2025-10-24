use paginator_rs::{PaginationParams, PaginatorError, PaginatorResponse};
use serde::Serialize;
use sqlx::{Database, Executor, FromRow};
use std::marker::PhantomData;

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
