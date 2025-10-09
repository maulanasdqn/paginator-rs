//! SurrealDB integration for paginator-rs
//!
//! This crate provides pagination support for SurrealDB queries with its multi-model approach.
//!
//! # Example
//!
//! ```ignore
//! use surrealdb::Surreal;
//! use surrealdb::engine::remote::ws::Ws;
//! use paginator_surrealdb::paginate_query;
//! use paginator_rs::PaginatorBuilder;
//!
//! #[derive(serde::Deserialize, serde::Serialize)]
//! struct User {
//!     id: String,
//!     name: String,
//!     email: String,
//! }
//!
//! async fn get_users() -> Result<(), Box<dyn std::error::Error>> {
//!     let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
//!     db.use_ns("test").use_db("test").await?;
//!
//!     let params = PaginatorBuilder::new()
//!         .page(1)
//!         .per_page(10)
//!         .sort_by("name")
//!         .sort_asc()
//!         .build();
//!
//!     let result = paginate_query::<User>(
//!         &db,
//!         "SELECT * FROM users WHERE active = true",
//!         &params,
//!     ).await?;
//!
//!     println!("Total: {}", result.meta.total);
//!     Ok(())
//! }
//! ```

use paginator_rs::{PaginationParams, PaginatorError, PaginatorResponse, PaginatorResponseMeta};
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::{Connection, Surreal};

/// Paginate a SurrealDB query with automatic counting and sorting
///
/// This function automatically adds LIMIT, START, and ORDER BY clauses to your query.
///
/// # Arguments
///
/// * `db` - Reference to the SurrealDB connection
/// * `base_query` - The base SQL query (without LIMIT/START/ORDER BY)
/// * `params` - Pagination parameters
///
/// # Example
///
/// ```ignore
/// let result = paginate_query::<User>(
///     &db,
///     "SELECT * FROM users WHERE status = 'active'",
///     &params,
/// ).await?;
/// ```
pub async fn paginate_query<T, C>(
    db: &Surreal<C>,
    base_query: &str,
    params: &PaginationParams,
) -> Result<PaginatorResponse<T>, PaginatorError>
where
    T: DeserializeOwned + Serialize,
    C: Connection,
{
    // Build count query
    // We need to get the total count first
    let mut count_query = if base_query.trim().to_uppercase().starts_with("SELECT") {
        // Extract the FROM clause and WHERE conditions
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

    // Add filters and search to count query WHERE clause
    if let Some(where_clause) = params.to_surrealql_where() {
        let query_upper = count_query.to_uppercase();
        if query_upper.contains(" WHERE ") {
            count_query.push_str(&format!(" AND {}", where_clause));
        } else {
            count_query.push_str(&format!(" WHERE {}", where_clause));
        }
    }

    // Execute count query
    let count_result: Vec<CountResult> = db
        .query(&count_query)
        .await
        .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?
        .take(0)
        .map_err(|e| PaginatorError::Custom(format!("Failed to extract count: {}", e)))?;

    let total = count_result
        .first()
        .map(|r| r.count as u32)
        .unwrap_or(0);

    // Build paginated query
    let mut paginated_query = base_query.to_string();

    // Add filters and search to WHERE clause
    if let Some(where_clause) = params.to_surrealql_where() {
        // Check if query already has a WHERE clause
        let query_upper = paginated_query.to_uppercase();
        if query_upper.contains(" WHERE ") {
            // Append to existing WHERE clause with AND
            paginated_query.push_str(&format!(" AND {}", where_clause));
        } else {
            // Add new WHERE clause
            paginated_query.push_str(&format!(" WHERE {}", where_clause));
        }
    }

    // Add ORDER BY if specified
    if let Some(ref sort_field) = params.sort_by {
        let direction = match params.sort_direction.as_ref() {
            Some(paginator_rs::SortDirection::Desc) => "DESC",
            _ => "ASC",
        };
        paginated_query.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
    }

    // Add LIMIT and START (SurrealDB uses START instead of OFFSET)
    paginated_query.push_str(&format!(
        " LIMIT {} START {}",
        params.limit(),
        params.offset()
    ));

    // Execute paginated query
    let data: Vec<T> = db
        .query(&paginated_query)
        .await
        .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?
        .take(0)
        .map_err(|e| PaginatorError::Custom(format!("Failed to extract results: {}", e)))?;

    Ok(PaginatorResponse {
        data,
        meta: PaginatorResponseMeta::new(params.page, params.per_page, total),
    })
}

/// Helper struct for counting results
#[derive(Debug, serde::Deserialize)]
struct CountResult {
    count: i64,
}

/// Paginate a table directly with optional filtering
///
/// This is a convenience function for simple table pagination.
///
/// # Example
///
/// ```ignore
/// let result = paginate_table::<User>(
///     &db,
///     "users",
///     Some("status = 'active'"),
///     &params,
/// ).await?;
/// ```
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

/// Paginate with a custom record ID range
///
/// Useful for cursor-based pagination in SurrealDB
///
/// # Example
///
/// ```ignore
/// let result = paginate_by_id_range::<User>(
///     &db,
///     "users",
///     Some("user:100"),
///     Some("user:200"),
///     &params,
/// ).await?;
/// ```
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

    paginate_table(
        db,
        table,
        where_clause.as_deref(),
        params,
    )
    .await
}

/// Builder for complex SurrealDB queries with pagination
///
/// # Example
///
/// ```ignore
/// let result = QueryBuilder::new()
///     .select("*")
///     .from("users")
///     .where_clause("age > 18")
///     .and("status = 'active'")
///     .paginate(&db, &params)
///     .await?;
/// ```
pub struct QueryBuilder {
    select: String,
    from: Option<String>,
    conditions: Vec<String>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            select: "*".to_string(),
            from: None,
            conditions: Vec::new(),
        }
    }

    /// Set the SELECT clause
    pub fn select(mut self, fields: &str) -> Self {
        self.select = fields.to_string();
        self
    }

    /// Set the FROM clause
    pub fn from(mut self, table: &str) -> Self {
        self.from = Some(table.to_string());
        self
    }

    /// Add a WHERE condition
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    /// Add an AND condition
    pub fn and(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    /// Build the query string
    fn build_query(&self) -> Result<String, PaginatorError> {
        let from = self
            .from
            .as_ref()
            .ok_or_else(|| PaginatorError::Custom("FROM clause is required".to_string()))?;

        let mut query = format!("SELECT {} FROM {}", self.select, from);

        if !self.conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", self.conditions.join(" AND ")));
        }

        Ok(query)
    }

    /// Execute the paginated query
    pub async fn paginate<T, C>(
        self,
        db: &Surreal<C>,
        params: &PaginationParams,
    ) -> Result<PaginatorResponse<T>, PaginatorError>
    where
        T: DeserializeOwned + Serialize,
        C: Connection,
    {
        let query = self.build_query()?;
        paginate_query(db, &query, params).await
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let builder = QueryBuilder::new()
            .select("id, name")
            .from("users")
            .where_clause("age > 18")
            .and("status = 'active'");

        let query = builder.build_query().unwrap();
        assert_eq!(
            query,
            "SELECT id, name FROM users WHERE age > 18 AND status = 'active'"
        );
    }

    #[test]
    fn test_query_builder_no_conditions() {
        let builder = QueryBuilder::new()
            .select("*")
            .from("users");

        let query = builder.build_query().unwrap();
        assert_eq!(query, "SELECT * FROM users");
    }

    #[test]
    fn test_query_builder_no_from() {
        let builder = QueryBuilder::new().select("*");

        let result = builder.build_query();
        assert!(result.is_err());
    }
}
