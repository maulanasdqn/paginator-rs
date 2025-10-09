use paginator_utils;
pub use paginator_utils::*;

use serde::Serialize;
use serde_json::{Value, to_value};
use std::error::Error;
use std::fmt;

/// Errors that can occur during pagination
#[derive(Debug)]
pub enum PaginatorError {
    /// Invalid page number (must be >= 1)
    InvalidPage(u32),
    /// Invalid per_page value (must be between 1 and max limit)
    InvalidPerPage(u32),
    /// Serialization error
    SerializationError(String),
    /// Custom error with message
    Custom(String),
}

impl fmt::Display for PaginatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaginatorError::InvalidPage(page) => {
                write!(f, "Invalid page number: {}. Page must be >= 1", page)
            }
            PaginatorError::InvalidPerPage(per_page) => {
                write!(
                    f,
                    "Invalid per_page value: {}. Must be between 1 and 100",
                    per_page
                )
            }
            PaginatorError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            PaginatorError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for PaginatorError {}

/// Result type for pagination operations
pub type PaginatorResult<T> = Result<T, PaginatorError>;

/// Main trait for implementing pagination on collections
pub trait PaginatorTrait<T>
where
    T: Serialize,
{
    /// Paginate the collection with given parameters
    fn paginate(&self, params: &PaginationParams) -> PaginatorResult<PaginatorResponse<T>> {
        // Validate parameters
        if params.page < 1 {
            return Err(PaginatorError::InvalidPage(params.page));
        }
        if params.per_page < 1 || params.per_page > 100 {
            return Err(PaginatorError::InvalidPerPage(params.per_page));
        }

        Ok(PaginatorResponse {
            data: vec![],
            meta: PaginatorResponseMeta::new(0, params.per_page, 0),
        })
    }

    /// Paginate and return as JSON Value
    fn paginate_json(&self, params: &PaginationParams) -> PaginatorResult<Value> {
        let response = self.paginate(params)?;
        to_value(response)
            .map_err(|e| PaginatorError::SerializationError(e.to_string()))
    }
}

/// Builder for creating pagination parameters with fluent API
pub struct PaginatorBuilder {
    params: PaginationParams,
}

impl Default for PaginatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginatorBuilder {
    /// Create a new builder with default parameters
    pub fn new() -> Self {
        Self {
            params: PaginationParams::default(),
        }
    }

    /// Set the page number (1-indexed)
    pub fn page(mut self, page: u32) -> Self {
        self.params.page = page.max(1);
        self
    }

    /// Set the number of items per page
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.params.per_page = per_page.max(1).min(100);
        self
    }

    /// Set the sorting field
    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.params.sort_by = Some(field.into());
        self
    }

    /// Set ascending sort direction
    pub fn sort_asc(mut self) -> Self {
        self.params.sort_direction = Some(SortDirection::Asc);
        self
    }

    /// Set descending sort direction
    pub fn sort_desc(mut self) -> Self {
        self.params.sort_direction = Some(SortDirection::Desc);
        self
    }

    /// Add a custom filter
    pub fn filter(mut self, field: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        self.params.filters.push(Filter::new(field, operator, value));
        self
    }

    /// Add an equals filter (field = value)
    pub fn filter_eq(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params.filters.push(Filter::new(field, FilterOperator::Eq, value));
        self
    }

    /// Add a not equals filter (field != value)
    pub fn filter_ne(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params.filters.push(Filter::new(field, FilterOperator::Ne, value));
        self
    }

    /// Add a greater than filter (field > value)
    pub fn filter_gt(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params.filters.push(Filter::new(field, FilterOperator::Gt, value));
        self
    }

    /// Add a less than filter (field < value)
    pub fn filter_lt(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params.filters.push(Filter::new(field, FilterOperator::Lt, value));
        self
    }

    /// Add a greater than or equal filter (field >= value)
    pub fn filter_gte(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params.filters.push(Filter::new(field, FilterOperator::Gte, value));
        self
    }

    /// Add a less than or equal filter (field <= value)
    pub fn filter_lte(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params.filters.push(Filter::new(field, FilterOperator::Lte, value));
        self
    }

    /// Add a LIKE filter (field LIKE pattern)
    pub fn filter_like(mut self, field: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::Like,
            FilterValue::String(pattern.into()),
        ));
        self
    }

    /// Add a case-insensitive LIKE filter (field ILIKE pattern)
    pub fn filter_ilike(mut self, field: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::ILike,
            FilterValue::String(pattern.into()),
        ));
        self
    }

    /// Add an IN filter (field IN array)
    pub fn filter_in(mut self, field: impl Into<String>, values: Vec<FilterValue>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::In,
            FilterValue::Array(values),
        ));
        self
    }

    /// Add a BETWEEN filter (field BETWEEN min AND max)
    pub fn filter_between(mut self, field: impl Into<String>, min: FilterValue, max: FilterValue) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::Between,
            FilterValue::Array(vec![min, max]),
        ));
        self
    }

    /// Add an IS NULL filter
    pub fn filter_is_null(mut self, field: impl Into<String>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::IsNull,
            FilterValue::Null,
        ));
        self
    }

    /// Add an IS NOT NULL filter
    pub fn filter_is_not_null(mut self, field: impl Into<String>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::IsNotNull,
            FilterValue::Null,
        ));
        self
    }

    /// Add search parameters
    pub fn search(mut self, query: impl Into<String>, fields: Vec<String>) -> Self {
        self.params.search = Some(SearchParams::new(query, fields));
        self
    }

    /// Add exact match search
    pub fn search_exact(mut self, query: impl Into<String>, fields: Vec<String>) -> Self {
        self.params.search = Some(SearchParams::new(query, fields).with_exact_match(true));
        self
    }

    /// Add case-sensitive search
    pub fn search_case_sensitive(mut self, query: impl Into<String>, fields: Vec<String>) -> Self {
        self.params.search = Some(SearchParams::new(query, fields).with_case_sensitive(true));
        self
    }

    /// Build the final PaginationParams
    pub fn build(self) -> PaginationParams {
        self.params
    }
}
