//! SQLx integration for paginator-rs
//!
//! This crate provides pagination support for SQLx queries across PostgreSQL, MySQL, and SQLite.
//!
//! # Example
//!
//! ```ignore
//! use sqlx::PgPool;
//! use paginator_sqlx::PaginateQuery;
//! use paginator_rs::PaginatorBuilder;
//!
//! #[derive(sqlx::FromRow, serde::Serialize)]
//! struct User {
//!     id: i32,
//!     name: String,
//! }
//!
//! async fn get_users(pool: &PgPool) -> Result<(), sqlx::Error> {
//!     let params = PaginatorBuilder::new()
//!         .page(1)
//!         .per_page(10)
//!         .build();
//!
//!     let result = sqlx::query_as::<_, User>("SELECT id, name FROM users")
//!         .paginate(&params)
//!         .fetch(pool)
//!         .await?;
//!
//!     println!("Users: {:?}", result);
//!     Ok(())
//! }
//! ```

use paginator_rs::{PaginationParams, PaginatorError, PaginatorResponse};
use serde::Serialize;
use sqlx::{Database, Executor, FromRow};
use std::marker::PhantomData;

/// Extension trait for paginating SQLx queries
pub trait PaginateQuery<'q, DB: Database, T>
where
    T: Send + Unpin,
{
    /// Paginate this query with the given parameters
    fn paginate(self, params: &PaginationParams) -> PaginatedQuery<'q, DB, T>;
}

/// A paginated SQLx query
pub struct PaginatedQuery<'q, DB: Database, T> {
    query: &'q str,
    _phantom: PhantomData<(DB, T)>,
}

impl<'q, DB: Database, T> PaginatedQuery<'q, DB, T> {
    /// Create a new paginated query
    pub fn new(query: &'q str, _params: &PaginationParams) -> Self {
        Self {
            query,
            _phantom: PhantomData,
        }
    }

    /// Execute the paginated query and return results
    pub async fn fetch<'e, E>(
        self,
        _executor: E,
    ) -> Result<PaginatorResponse<T>, PaginatorError>
    where
        E: Executor<'e, Database = DB>,
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin + Serialize,
    {
        // First, get the total count
        let _count_query = format!("SELECT COUNT(*) FROM ({})", self.query);

        // Note: This is a simplified version. In production, you'd need to handle
        // different database types and properly bind parameters

        // For now, return a placeholder error indicating this needs database-specific implementation
        Err(PaginatorError::Custom(
            "SQLx integration requires database-specific query building. \
             See examples for proper implementation.".to_string()
        ))
    }
}

// PostgreSQL-specific implementation
#[cfg(feature = "postgres")]
pub mod postgres {
    use super::*;
    use sqlx::postgres::{PgArguments, PgRow};
    use sqlx::{Postgres, query::Query};

    impl<'q, T> PaginateQuery<'q, Postgres, T> for Query<'q, Postgres, PgArguments>
    where
        T: Send + Unpin,
    {
        fn paginate(self, params: &PaginationParams) -> PaginatedQuery<'q, Postgres, T> {
            PaginatedQuery::new("", params)
        }
    }

    /// Paginate a raw SQL query for PostgreSQL
    pub async fn paginate_query<'e, E, T>(
        executor: E,
        base_query: &str,
        params: &PaginationParams,
    ) -> Result<PaginatorResponse<T>, PaginatorError>
    where
        E: Executor<'e, Database = Postgres>,
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin + Serialize,
    {
        // Apply filters and search to base query
        let mut modified_query = base_query.to_string();

        if let Some(where_clause) = params.to_sql_where() {
            let query_upper = modified_query.to_uppercase();
            if query_upper.contains(" WHERE ") {
                modified_query.push_str(&format!(" AND {}", where_clause));
            } else {
                modified_query.push_str(&format!(" WHERE {}", where_clause));
            }
        }

        // Build count query
        let count_query = format!("SELECT COUNT(*) FROM ({}) as count_subquery", modified_query);
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(executor)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;

        // Build paginated query
        let mut paginated_query = modified_query;

        // Add ORDER BY if specified
        if let Some(ref sort_field) = params.sort_by {
            let direction = params
                .sort_direction
                .as_ref()
                .map(|d| format!("{:?}", d).to_uppercase())
                .unwrap_or_else(|| "ASC".to_string());
            paginated_query.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
        }

        // Add LIMIT and OFFSET
        paginated_query.push_str(&format!(" LIMIT {} OFFSET {}", params.limit(), params.offset()));

        // Execute paginated query
        let data: Vec<T> = sqlx::query_as(&paginated_query)
            .fetch_all(executor)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?;

        Ok(PaginatorResponse {
            data,
            meta: PaginatorResponseMeta::new(params.page, params.per_page, total.0 as u32),
        })
    }
}

// MySQL-specific implementation
#[cfg(feature = "mysql")]
pub mod mysql {
    use super::*;
    use sqlx::mysql::{MySqlArguments, MySqlRow};
    use sqlx::{MySql, query::Query};

    impl<'q, T> PaginateQuery<'q, MySql, T> for Query<'q, MySql, MySqlArguments>
    where
        T: Send + Unpin,
    {
        fn paginate(self, params: &PaginationParams) -> PaginatedQuery<'q, MySql, T> {
            PaginatedQuery::new("", params)
        }
    }

    /// Paginate a raw SQL query for MySQL
    pub async fn paginate_query<'e, E, T>(
        executor: E,
        base_query: &str,
        params: &PaginationParams,
    ) -> Result<PaginatorResponse<T>, PaginatorError>
    where
        E: Executor<'e, Database = MySql>,
        T: for<'r> FromRow<'r, MySqlRow> + Send + Unpin + Serialize,
    {
        // Apply filters and search to base query
        let mut modified_query = base_query.to_string();

        if let Some(where_clause) = params.to_sql_where() {
            let query_upper = modified_query.to_uppercase();
            if query_upper.contains(" WHERE ") {
                modified_query.push_str(&format!(" AND {}", where_clause));
            } else {
                modified_query.push_str(&format!(" WHERE {}", where_clause));
            }
        }

        // Build count query
        let count_query = format!("SELECT COUNT(*) FROM ({}) as count_subquery", modified_query);
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(executor)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;

        // Build paginated query
        let mut paginated_query = modified_query;

        // Add ORDER BY if specified
        if let Some(ref sort_field) = params.sort_by {
            let direction = params
                .sort_direction
                .as_ref()
                .map(|d| format!("{:?}", d).to_uppercase())
                .unwrap_or_else(|| "ASC".to_string());
            paginated_query.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
        }

        // Add LIMIT and OFFSET
        paginated_query.push_str(&format!(" LIMIT {} OFFSET {}", params.limit(), params.offset()));

        // Execute paginated query
        let data: Vec<T> = sqlx::query_as(&paginated_query)
            .fetch_all(executor)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?;

        Ok(PaginatorResponse {
            data,
            meta: PaginatorResponseMeta::new(params.page, params.per_page, total.0 as u32),
        })
    }
}

// SQLite-specific implementation
#[cfg(feature = "sqlite")]
pub mod sqlite {
    use super::*;
    use sqlx::sqlite::{SqliteArguments, SqliteRow};
    use sqlx::{Sqlite, query::Query};

    impl<'q, T> PaginateQuery<'q, Sqlite, T> for Query<'q, Sqlite, SqliteArguments>
    where
        T: Send + Unpin,
    {
        fn paginate(self, params: &PaginationParams) -> PaginatedQuery<'q, Sqlite, T> {
            PaginatedQuery::new("", params)
        }
    }

    /// Paginate a raw SQL query for SQLite
    pub async fn paginate_query<'e, E, T>(
        executor: E,
        base_query: &str,
        params: &PaginationParams,
    ) -> Result<PaginatorResponse<T>, PaginatorError>
    where
        E: Executor<'e, Database = Sqlite>,
        T: for<'r> FromRow<'r, SqliteRow> + Send + Unpin + Serialize,
    {
        // Apply filters and search to base query
        let mut modified_query = base_query.to_string();

        if let Some(where_clause) = params.to_sql_where() {
            let query_upper = modified_query.to_uppercase();
            if query_upper.contains(" WHERE ") {
                modified_query.push_str(&format!(" AND {}", where_clause));
            } else {
                modified_query.push_str(&format!(" WHERE {}", where_clause));
            }
        }

        // Build count query
        let count_query = format!("SELECT COUNT(*) FROM ({}) as count_subquery", modified_query);
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(executor)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;

        // Build paginated query
        let mut paginated_query = modified_query;

        // Add ORDER BY if specified
        if let Some(ref sort_field) = params.sort_by {
            let direction = params
                .sort_direction
                .as_ref()
                .map(|d| format!("{:?}", d).to_uppercase())
                .unwrap_or_else(|| "ASC".to_string());
            paginated_query.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
        }

        // Add LIMIT and OFFSET
        paginated_query.push_str(&format!(" LIMIT {} OFFSET {}", params.limit(), params.offset()));

        // Execute paginated query
        let data: Vec<T> = sqlx::query_as(&paginated_query)
            .fetch_all(executor)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?;

        Ok(PaginatorResponse {
            data,
            meta: PaginatorResponseMeta::new(params.page, params.per_page, total.0 as u32),
        })
    }
}
