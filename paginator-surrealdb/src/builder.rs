use crate::query::paginate_query;
use paginator_rs::{PaginationParams, PaginatorError, PaginatorResponse};
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::{Connection, Surreal};

pub struct QueryBuilder {
    select: String,
    from: Option<String>,
    conditions: Vec<String>,
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            select: "*".to_string(),
            from: None,
            conditions: Vec::new(),
        }
    }

    pub fn select(mut self, fields: &str) -> Self {
        self.select = fields.to_string();
        self
    }

    pub fn from(mut self, table: &str) -> Self {
        self.from = Some(table.to_string());
        self
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    pub fn and(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    pub fn build_query(&self) -> Result<String, PaginatorError> {
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
