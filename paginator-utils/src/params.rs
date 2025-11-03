use crate::cursor::Cursor;
use crate::filter::Filter;
use crate::search::SearchParams;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub per_page: u32,
    pub sort_by: Option<String>,
    pub sort_direction: Option<SortDirection>,
    #[serde(default)]
    pub filters: Vec<Filter>,
    pub search: Option<SearchParams>,
    #[serde(default)]
    pub disable_total_count: bool,
    pub cursor: Option<Cursor>,
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
            disable_total_count: false,
            cursor: None,
        }
    }
}

impl PaginationParams {
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
            sort_by: None,
            sort_direction: None,
            filters: Vec::new(),
            search: None,
            disable_total_count: false,
            cursor: None,
        }
    }

    pub fn with_sort(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }

    pub fn with_direction(mut self, direction: SortDirection) -> Self {
        self.sort_direction = Some(direction);
        self
    }

    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn with_filters(mut self, filters: Vec<Filter>) -> Self {
        self.filters.extend(filters);
        self
    }

    pub fn with_search(mut self, search: SearchParams) -> Self {
        self.search = Some(search);
        self
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    pub fn limit(&self) -> u32 {
        self.per_page
    }

    pub fn to_sql_where(&self) -> Option<String> {
        let mut conditions = Vec::new();

        for filter in &self.filters {
            conditions.push(filter.to_sql_where());
        }

        if let Some(ref search) = self.search {
            conditions.push(search.to_sql_where());
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }

    pub fn to_surrealql_where(&self) -> Option<String> {
        let mut conditions = Vec::new();

        for filter in &self.filters {
            conditions.push(filter.to_surrealql_where());
        }

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
