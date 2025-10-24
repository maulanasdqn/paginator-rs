use paginator_utils::{Cursor, CursorDirection, CursorValue, Filter, FilterOperator, FilterValue, PaginationParams, SearchParams, SortDirection};

pub struct PaginatorBuilder {
    params: PaginationParams,
}

impl Default for PaginatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginatorBuilder {
    pub fn new() -> Self {
        Self {
            params: PaginationParams::default(),
        }
    }

    pub fn page(mut self, page: u32) -> Self {
        self.params.page = page.max(1);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.params.per_page = per_page.max(1).min(100);
        self
    }

    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.params.sort_by = Some(field.into());
        self
    }

    pub fn sort_asc(mut self) -> Self {
        self.params.sort_direction = Some(SortDirection::Asc);
        self
    }

    pub fn sort_desc(mut self) -> Self {
        self.params.sort_direction = Some(SortDirection::Desc);
        self
    }

    pub fn filter(
        mut self,
        field: impl Into<String>,
        operator: FilterOperator,
        value: FilterValue,
    ) -> Self {
        self.params
            .filters
            .push(Filter::new(field, operator, value));
        self
    }

    pub fn filter_eq(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params
            .filters
            .push(Filter::new(field, FilterOperator::Eq, value));
        self
    }

    pub fn filter_ne(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params
            .filters
            .push(Filter::new(field, FilterOperator::Ne, value));
        self
    }

    pub fn filter_gt(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params
            .filters
            .push(Filter::new(field, FilterOperator::Gt, value));
        self
    }

    pub fn filter_lt(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params
            .filters
            .push(Filter::new(field, FilterOperator::Lt, value));
        self
    }

    pub fn filter_gte(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params
            .filters
            .push(Filter::new(field, FilterOperator::Gte, value));
        self
    }

    pub fn filter_lte(mut self, field: impl Into<String>, value: FilterValue) -> Self {
        self.params
            .filters
            .push(Filter::new(field, FilterOperator::Lte, value));
        self
    }

    pub fn filter_like(mut self, field: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::Like,
            FilterValue::String(pattern.into()),
        ));
        self
    }

    pub fn filter_ilike(mut self, field: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::ILike,
            FilterValue::String(pattern.into()),
        ));
        self
    }

    pub fn filter_in(mut self, field: impl Into<String>, values: Vec<FilterValue>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::In,
            FilterValue::Array(values),
        ));
        self
    }

    pub fn filter_between(
        mut self,
        field: impl Into<String>,
        min: FilterValue,
        max: FilterValue,
    ) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::Between,
            FilterValue::Array(vec![min, max]),
        ));
        self
    }

    pub fn filter_is_null(mut self, field: impl Into<String>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::IsNull,
            FilterValue::Null,
        ));
        self
    }

    pub fn filter_is_not_null(mut self, field: impl Into<String>) -> Self {
        self.params.filters.push(Filter::new(
            field,
            FilterOperator::IsNotNull,
            FilterValue::Null,
        ));
        self
    }

    pub fn search(mut self, query: impl Into<String>, fields: Vec<String>) -> Self {
        self.params.search = Some(SearchParams::new(query, fields));
        self
    }

    pub fn search_exact(mut self, query: impl Into<String>, fields: Vec<String>) -> Self {
        self.params.search = Some(SearchParams::new(query, fields).with_exact_match(true));
        self
    }

    pub fn search_case_sensitive(mut self, query: impl Into<String>, fields: Vec<String>) -> Self {
        self.params.search = Some(SearchParams::new(query, fields).with_case_sensitive(true));
        self
    }

    pub fn disable_total_count(mut self) -> Self {
        self.params.disable_total_count = true;
        self
    }

    pub fn cursor(mut self, field: impl Into<String>, value: CursorValue, direction: CursorDirection) -> Self {
        self.params.cursor = Some(Cursor::new(field.into(), value, direction));
        self
    }

    pub fn cursor_after(mut self, field: impl Into<String>, value: CursorValue) -> Self {
        self.params.cursor = Some(Cursor::new(field.into(), value, CursorDirection::After));
        self
    }

    pub fn cursor_before(mut self, field: impl Into<String>, value: CursorValue) -> Self {
        self.params.cursor = Some(Cursor::new(field.into(), value, CursorDirection::Before));
        self
    }

    pub fn cursor_from_encoded(mut self, encoded: &str) -> Result<Self, String> {
        self.params.cursor = Some(Cursor::decode(encoded)?);
        Ok(self)
    }

    pub fn build(self) -> PaginationParams {
        self.params
    }
}
