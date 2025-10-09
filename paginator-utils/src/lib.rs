use serde::{Deserialize, Serialize};

/// Filter operators for querying data
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FilterOperator {
    /// Equal to (=)
    Eq,
    /// Not equal to (!=)
    Ne,
    /// Greater than (>)
    Gt,
    /// Less than (<)
    Lt,
    /// Greater than or equal to (>=)
    Gte,
    /// Less than or equal to (<=)
    Lte,
    /// SQL LIKE pattern matching
    Like,
    /// Case-insensitive LIKE (PostgreSQL ILIKE)
    ILike,
    /// Value in array (IN)
    In,
    /// Value not in array (NOT IN)
    NotIn,
    /// Is NULL
    IsNull,
    /// Is NOT NULL
    IsNotNull,
    /// Between two values
    Between,
    /// Contains (for arrays/JSON)
    Contains,
}

/// Value types for filters
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FilterValue {
    /// String value
    String(String),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Array of values
    Array(Vec<FilterValue>),
    /// Null value
    Null,
}

impl FilterValue {
    /// Convert to SQL string representation
    pub fn to_sql_string(&self) -> String {
        match self {
            FilterValue::String(s) => format!("'{}'", s.replace('\'', "''")),
            FilterValue::Int(i) => i.to_string(),
            FilterValue::Float(f) => f.to_string(),
            FilterValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            FilterValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_sql_string()).collect();
                format!("({})", items.join(", "))
            }
            FilterValue::Null => "NULL".to_string(),
        }
    }
}

/// A single filter condition
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Filter {
    /// Field name to filter on
    pub field: String,
    /// Filter operator
    pub operator: FilterOperator,
    /// Filter value
    pub value: FilterValue,
}

impl Filter {
    /// Create a new filter
    pub fn new(field: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
        }
    }

    /// Convert filter to SQL WHERE clause fragment
    pub fn to_sql_where(&self) -> String {
        match &self.operator {
            FilterOperator::Eq => format!("{} = {}", self.field, self.value.to_sql_string()),
            FilterOperator::Ne => format!("{} != {}", self.field, self.value.to_sql_string()),
            FilterOperator::Gt => format!("{} > {}", self.field, self.value.to_sql_string()),
            FilterOperator::Lt => format!("{} < {}", self.field, self.value.to_sql_string()),
            FilterOperator::Gte => format!("{} >= {}", self.field, self.value.to_sql_string()),
            FilterOperator::Lte => format!("{} <= {}", self.field, self.value.to_sql_string()),
            FilterOperator::Like => format!("{} LIKE {}", self.field, self.value.to_sql_string()),
            FilterOperator::ILike => format!("{} ILIKE {}", self.field, self.value.to_sql_string()),
            FilterOperator::In => format!("{} IN {}", self.field, self.value.to_sql_string()),
            FilterOperator::NotIn => format!("{} NOT IN {}", self.field, self.value.to_sql_string()),
            FilterOperator::IsNull => format!("{} IS NULL", self.field),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", self.field),
            FilterOperator::Between => {
                if let FilterValue::Array(arr) = &self.value {
                    if arr.len() == 2 {
                        return format!(
                            "{} BETWEEN {} AND {}",
                            self.field,
                            arr[0].to_sql_string(),
                            arr[1].to_sql_string()
                        );
                    }
                }
                format!("{} = {}", self.field, self.value.to_sql_string())
            }
            FilterOperator::Contains => {
                format!("{} @> {}", self.field, self.value.to_sql_string())
            }
        }
    }

    /// Convert filter to SurrealQL WHERE clause fragment
    pub fn to_surrealql_where(&self) -> String {
        match &self.operator {
            FilterOperator::Eq => format!("{} = {}", self.field, self.value.to_sql_string()),
            FilterOperator::Ne => format!("{} != {}", self.field, self.value.to_sql_string()),
            FilterOperator::Gt => format!("{} > {}", self.field, self.value.to_sql_string()),
            FilterOperator::Lt => format!("{} < {}", self.field, self.value.to_sql_string()),
            FilterOperator::Gte => format!("{} >= {}", self.field, self.value.to_sql_string()),
            FilterOperator::Lte => format!("{} <= {}", self.field, self.value.to_sql_string()),
            FilterOperator::Like | FilterOperator::ILike => {
                format!("{} ~ {}", self.field, self.value.to_sql_string())
            }
            FilterOperator::In => format!("{} INSIDE {}", self.field, self.value.to_sql_string()),
            FilterOperator::NotIn => format!("{} NOT INSIDE {}", self.field, self.value.to_sql_string()),
            FilterOperator::IsNull => format!("{} IS NULL", self.field),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", self.field),
            FilterOperator::Between => {
                if let FilterValue::Array(arr) = &self.value {
                    if arr.len() == 2 {
                        return format!(
                            "{} >= {} AND {} <= {}",
                            self.field,
                            arr[0].to_sql_string(),
                            self.field,
                            arr[1].to_sql_string()
                        );
                    }
                }
                format!("{} = {}", self.field, self.value.to_sql_string())
            }
            FilterOperator::Contains => {
                format!("{} CONTAINS {}", self.field, self.value.to_sql_string())
            }
        }
    }
}

/// Search parameters for full-text search
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchParams {
    /// Search query text
    pub query: String,
    /// Fields to search in
    pub fields: Vec<String>,
    /// Case-sensitive search (default: false)
    #[serde(default)]
    pub case_sensitive: bool,
    /// Exact match (default: false, uses fuzzy/LIKE)
    #[serde(default)]
    pub exact_match: bool,
}

impl SearchParams {
    /// Create new search parameters
    pub fn new(query: impl Into<String>, fields: Vec<String>) -> Self {
        Self {
            query: query.into(),
            fields,
            case_sensitive: false,
            exact_match: false,
        }
    }

    /// Set case sensitivity
    pub fn with_case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Set exact match mode
    pub fn with_exact_match(mut self, exact: bool) -> Self {
        self.exact_match = exact;
        self
    }

    /// Generate SQL WHERE clause for search
    pub fn to_sql_where(&self) -> String {
        let pattern = if self.exact_match {
            format!("'{}'", self.query.replace('\'', "''"))
        } else {
            format!("'%{}%'", self.query.replace('\'', "''"))
        };

        let operator = if self.case_sensitive {
            "LIKE"
        } else {
            "ILIKE" // PostgreSQL, falls back to LOWER() for others
        };

        let conditions: Vec<String> = self
            .fields
            .iter()
            .map(|field| {
                if self.case_sensitive || operator == "ILIKE" {
                    format!("{} {} {}", field, operator, pattern)
                } else {
                    format!("LOWER({}) LIKE LOWER({})", field, pattern)
                }
            })
            .collect();

        format!("({})", conditions.join(" OR "))
    }
}

/// Pagination parameters for controlling page size and navigation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Current page number (1-indexed)
    pub page: u32,
    /// Number of items per page
    pub per_page: u32,
    /// Optional sorting field
    pub sort_by: Option<String>,
    /// Sort direction: "asc" or "desc"
    pub sort_direction: Option<SortDirection>,
    /// Filters to apply
    #[serde(default)]
    pub filters: Vec<Filter>,
    /// Search parameters
    pub search: Option<SearchParams>,
}

/// Sort direction for ordering results
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
            sort_by: None,
            sort_direction: None,
            filters: Vec::new(),
            search: None,
        }
    }
}

impl PaginationParams {
    /// Create new pagination parameters
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.max(1).min(100),
            sort_by: None,
            sort_direction: None,
            filters: Vec::new(),
            search: None,
        }
    }

    /// Set sorting field
    pub fn with_sort(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }

    /// Set sort direction
    pub fn with_direction(mut self, direction: SortDirection) -> Self {
        self.sort_direction = Some(direction);
        self
    }

    /// Add a filter
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Add multiple filters
    pub fn with_filters(mut self, filters: Vec<Filter>) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Set search parameters
    pub fn with_search(mut self, search: SearchParams) -> Self {
        self.search = Some(search);
        self
    }

    /// Calculate offset for database queries
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    /// Get limit for database queries
    pub fn limit(&self) -> u32 {
        self.per_page
    }

    /// Generate SQL WHERE clause from filters and search
    pub fn to_sql_where(&self) -> Option<String> {
        let mut conditions = Vec::new();

        // Add filter conditions
        for filter in &self.filters {
            conditions.push(filter.to_sql_where());
        }

        // Add search condition
        if let Some(ref search) = self.search {
            conditions.push(search.to_sql_where());
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }

    /// Generate SurrealQL WHERE clause from filters and search
    pub fn to_surrealql_where(&self) -> Option<String> {
        let mut conditions = Vec::new();

        // Add filter conditions
        for filter in &self.filters {
            conditions.push(filter.to_surrealql_where());
        }

        // Add search condition (SurrealDB uses similar LIKE syntax)
        if let Some(ref search) = self.search {
            let search_conditions: Vec<String> = search
                .fields
                .iter()
                .map(|field| {
                    let pattern = if search.exact_match {
                        format!("'{}'", search.query.replace('\'', "''"))
                    } else {
                        format!("'%{}%'", search.query.replace('\'', "''"))
                    };
                    format!("{} ~ {}", field, pattern)
                })
                .collect();
            conditions.push(format!("({})", search_conditions.join(" OR ")));
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }
}

/// Paginated response containing data and metadata
#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatorResponse<T> {
    /// The paginated data items
    pub data: Vec<T>,
    /// Pagination metadata
    pub meta: PaginatorResponseMeta,
}

/// Metadata about the pagination state
#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatorResponseMeta {
    /// Current page number
    pub page: u32,
    /// Items per page
    pub per_page: u32,
    /// Total number of items
    pub total: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_prev: bool,
}

impl PaginatorResponseMeta {
    /// Create new metadata with computed fields
    pub fn new(page: u32, per_page: u32, total: u32) -> Self {
        let total_pages = (total as f32 / per_page as f32).ceil() as u32;
        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}
