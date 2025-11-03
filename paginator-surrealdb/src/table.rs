use crate::query::paginate_query;
use paginator_rs::{PaginationParams, PaginatorError, PaginatorResponse};
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::{Connection, Surreal};

pub async fn paginate_table<T, C>(
    db: &Surreal<C>,
    table: &str,
    where_clause: Option<&str>,
    params: &PaginationParams,
) -> Result<PaginatorResponse<T>, PaginatorError>
where
    T: DeserializeOwned + Serialize,
    C: Connection,
{
    let base_query = if let Some(condition) = where_clause {
        format!("SELECT * FROM {} WHERE {}", table, condition)
    } else {
        format!("SELECT * FROM {}", table)
    };

    paginate_query(db, &base_query, params).await
}

pub async fn paginate_by_id_range<T, C>(
    db: &Surreal<C>,
    table: &str,
    start_id: Option<&str>,
    end_id: Option<&str>,
    params: &PaginationParams,
) -> Result<PaginatorResponse<T>, PaginatorError>
where
    T: DeserializeOwned + Serialize,
    C: Connection,
{
    let mut conditions = Vec::new();

    if let Some(start) = start_id {
        conditions.push(format!("id >= {}", start));
    }

    if let Some(end) = end_id {
        conditions.push(format!("id <= {}", end));
    }

    let where_clause = if conditions.is_empty() {
        None
    } else {
        Some(conditions.join(" AND "))
    };

    paginate_table(db, table, where_clause.as_deref(), params).await
}
