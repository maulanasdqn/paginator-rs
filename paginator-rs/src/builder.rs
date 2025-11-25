use paginator_utils::{
    Cursor, CursorDirection, CursorValue, Filter, FilterOperator, FilterValue, PaginationParams,
    SearchParams, SortDirection, IntoPaginationParams,
};
use std::marker::PhantomData;

//
// ========================
//   PAGINATOR ROOT BUILDER
// ========================
//
pub struct Paginator<State = Ready> {
    params: PaginationParams,
    _state: PhantomData<State>,
}

/// Marker state
pub struct Ready;

impl Default for Paginator {
    fn default() -> Self {
        Self::new()
    }
}

impl Paginator {
    pub fn new() -> Self {
        Self {
            params: PaginationParams::default(),
            _state: PhantomData,
        }
    }

    //
    // -------------- BASIC CONFIG --------------
    //
    pub fn page(mut self, page: u32) -> Self {
        self.params.page = page.max(1);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.params.per_page = per_page.clamp(1, 100);
        self
    }

    //
    // -------------- SORT BUILDER --------------
    //
    pub fn sort(self) -> SortBuilder<Self> {
        SortBuilder::new(self)
    }

    //
    // -------------- FILTER BUILDER --------------
    //
    pub fn filter(self) -> FilterBuilder<Self> {
        FilterBuilder::with_parent(self)
    }

    //
    // -------------- SEARCH BUILDER --------------
    //
    pub fn search(self) -> SearchBuilder<Self> {
        SearchBuilder::with_parent(self)
    }

    //
    // -------------- CURSOR BUILDER --------------
    //
    pub fn cursor(self) -> CursorBuilder<Self> {
        CursorBuilder::with_parent(self)
    }

    pub fn disable_total_count(mut self) -> Self {
        self.params.disable_total_count = true;
        self
    }

    //
    // -------------- FINAL BUILD --------------
    //
    pub fn build(self) -> PaginationParams {
        self.params
    }
}

//
// ========================
//       SORT BUILDER
// ========================
//

pub struct SortBuilder<P> {
    parent: P,
}

impl<P> SortBuilder<P> {
    fn new(parent: P) -> Self {
        Self { parent }
    }

    pub fn asc(mut self, field: impl Into<String>) -> P
    where
        P: HasParams,
    {
        let mut p = self.parent;
        p.params_mut().sort_by = Some(field.into());
        p.params_mut().sort_direction = Some(SortDirection::Asc);
        p
    }

    pub fn desc(mut self, field: impl Into<String>) -> P
    where
        P: HasParams,
    {
        let mut p = self.parent;
        p.params_mut().sort_by = Some(field.into());
        p.params_mut().sort_direction = Some(SortDirection::Desc);
        p
    }
}

//
// ========================
//      FILTER BUILDER
// ========================
//

pub struct FilterBuilder<P = ()> {
    parent: Option<P>,
    filters: Vec<Filter>,
}

impl FilterBuilder<()> {
    /// Create as standalone (no parent)
    pub fn new() -> Self {
        Self {
            parent: None,
            filters: Vec::new(),
        }
    }

    /// Finish and return only the filters
    pub fn build(self) -> Vec<Filter> {
        self.filters
    }
}

impl<P> FilterBuilder<P> {
    /// Create with parent (Paginator, or any other)
    pub fn with_parent(parent: P) -> Self {
        Self {
            parent: Some(parent),
            filters: Vec::new(),
        }
    }

    // --- PRIMITIVES ---

    fn push(mut self, field: impl Into<String>, op: FilterOperator, value: FilterValue) -> Self {
        self.filters.push(Filter::new(field, op, value));
        self
    }

    pub fn eq(self, field: impl Into<String>, value: FilterValue) -> Self {
        self.push(field, FilterOperator::Eq, value)
    }

    pub fn ne(self, field: impl Into<String>, value: FilterValue) -> Self {
        self.push(field, FilterOperator::Ne, value)
    }

    pub fn gt(self, field: impl Into<String>, value: FilterValue) -> Self {
        self.push(field, FilterOperator::Gt, value)
    }

    pub fn lt(self, field: impl Into<String>, value: FilterValue) -> Self {
        self.push(field, FilterOperator::Lt, value)
    }

    pub fn gte(self, field: impl Into<String>, value: FilterValue) -> Self {
        self.push(field, FilterOperator::Gte, value)
    }

    pub fn lte(self, field: impl Into<String>, value: FilterValue) -> Self {
        self.push(field, FilterOperator::Lte, value)
    }

    pub fn like(self, field: impl Into<String>, pat: impl Into<String>) -> Self {
        self.push(field, FilterOperator::Like, FilterValue::String(pat.into()))
    }

    pub fn ilike(self, field: impl Into<String>, pat: impl Into<String>) -> Self {
        self.push(field, FilterOperator::ILike, FilterValue::String(pat.into()))
    }

    pub fn r#in(self, field: impl Into<String>, values: Vec<FilterValue>) -> Self {
        self.push(field, FilterOperator::In, FilterValue::Array(values))
    }

    pub fn not_in(self, field: impl Into<String>, values: Vec<FilterValue>) -> Self {
        self.push(field, FilterOperator::NotIn, FilterValue::Array(values))
    }

    pub fn between(self, field: impl Into<String>, min: FilterValue, max: FilterValue) -> Self {
        self.push(field, FilterOperator::Between, FilterValue::Array(vec![min, max]))
    }

    pub fn is_null(self, field: impl Into<String>) -> Self {
        self.push(field, FilterOperator::IsNull, FilterValue::Null)
    }

    pub fn is_not_null(self, field: impl Into<String>) -> Self {
        self.push(field, FilterOperator::IsNotNull, FilterValue::Null)
    }

    pub fn contains(self, field: impl Into<String>, value: FilterValue) -> Self {
        self.push(field, FilterOperator::Contains, value)
    }

    /// Finish and return to parent
    pub fn apply(self) -> P
    where
        P: HasParams,
    {
        let mut parent = self.parent.expect("FilterBuilder::apply called without a parent");
        parent.params_mut().filters.extend(self.filters);
        parent
    }
}

//
// ========================
//      SEARCH BUILDER
// ========================
//

pub struct SearchBuilder<P = ()> {
    parent: Option<P>,
    query: Option<String>,
    fields: Vec<String>,
    exact: bool,
    case_sensitive: bool,
}

impl SearchBuilder<()> {
    pub fn new() -> Self {
        Self {
            parent: None,
            query: None,
            fields: Vec::new(),
            exact: false,
            case_sensitive: false,
        }
    }

    pub fn build(self) -> Option<SearchParams> {
        self.query.map(|q| {
            let mut params = SearchParams::new(q, self.fields);
            if self.exact {
                params = params.with_exact_match(true);
            }
            if self.case_sensitive {
                params = params.with_case_sensitive(true);
            }
            params
        })
    }
}

impl<P> SearchBuilder<P> {
    pub fn with_parent(parent: P) -> Self {
        Self {
            parent: Some(parent),
            query: None,
            fields: Vec::new(),
            exact: false,
            case_sensitive: false,
        }
    }

    pub fn query(mut self, q: impl Into<String>) -> Self {
        self.query = Some(q.into());
        self
    }

    pub fn fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.fields = fields.into_iter().map(Into::into).collect();
        self
    }

    pub fn exact(mut self, yes: bool) -> Self {
        self.exact = yes;
        self
    }

    pub fn case_sensitive(mut self, yes: bool) -> Self {
        self.case_sensitive = yes;
        self
    }

    pub fn apply(self) -> P
    where
        P: HasParams,
    {
        let mut parent = self.parent.expect("SearchBuilder::apply called without a parent");

        if let Some(q) = self.query {
            let mut s = SearchParams::new(q, self.fields);
            if self.exact { s = s.with_exact_match(true); }
            if self.case_sensitive { s = s.with_case_sensitive(true); }
            parent.params_mut().search = Some(s);
        }

        parent
    }
}

//
// ========================
//      CURSOR BUILDER
// ========================
//

pub struct CursorBuilder<P = ()> {
    parent: Option<P>,
    cursor: Option<Cursor>,
}

impl CursorBuilder<()> {
    pub fn new() -> Self {
        Self {
            parent: None,
            cursor: None,
        }
    }

    pub fn build(self) -> Option<Cursor> {
        self.cursor
    }
}

impl<P> CursorBuilder<P> {
    pub fn with_parent(parent: P) -> Self {
        Self {
            parent: Some(parent),
            cursor: None,
        }
    }

    pub fn after(mut self, field: impl Into<String>, value: CursorValue) -> Self {
        self.cursor = Some(Cursor::new(field.into(), value, CursorDirection::After));
        self
    }

    pub fn before(mut self, field: impl Into<String>, value: CursorValue) -> Self {
        self.cursor = Some(Cursor::new(field.into(), value, CursorDirection::Before));
        self
    }

    pub fn from_encoded(mut self, encoded: &str) -> Result<Self, String> {
        self.cursor = Some(Cursor::decode(encoded)?);
        Ok(self)
    }

    pub fn apply(self) -> P
    where
        P: HasParams,
    {
        let mut parent = self.parent.expect("CursorBuilder::apply called without a parent");
        if let Some(cursor) = self.cursor {
            parent.params_mut().cursor = Some(cursor);
        }
        parent
    }
}

/// Trait for types that have params
pub trait HasParams {
    fn params_mut(&mut self) -> &mut PaginationParams;
}

impl<S> HasParams for Paginator<S> {
    fn params_mut(&mut self) -> &mut PaginationParams {
        &mut self.params
    }
}

impl<S> IntoPaginationParams for Paginator<S> {
    fn into_pagination_params(self) -> PaginationParams {
        self.params
    }
}

impl IntoPaginationParams for FilterBuilder<()> {
    fn into_pagination_params(self) -> PaginationParams {
        PaginationParams {
            filters: self.filters,
            ..Default::default()
        }
    }
}

impl IntoPaginationParams for SearchBuilder<()> {
    fn into_pagination_params(self) -> PaginationParams {
        let mut params = PaginationParams::default();
        if let Some(query) = self.query {
            let mut search = SearchParams::new(query, self.fields);
            if self.exact {
                search = search.with_exact_match(true);
            }
            if self.case_sensitive {
                search = search.with_case_sensitive(true);
            }
            params.search = Some(search);
        }
        params
    }
}

impl IntoPaginationParams for CursorBuilder<()> {
    fn into_pagination_params(self) -> PaginationParams {
        PaginationParams {
            cursor: self.cursor,
            ..Default::default()
        }
    }
}

impl IntoPaginationParams for PaginatorBuilder {
    fn into_pagination_params(self) -> PaginationParams {
        self.params
    }
}

//
// ========================
//   BACKWARD COMPATIBILITY
// ========================
//

/// Legacy builder for backward compatibility
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
        self.params.per_page = per_page.clamp(1, 100);
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

    pub fn cursor(
        mut self,
        field: impl Into<String>,
        value: CursorValue,
        direction: CursorDirection,
    ) -> Self {
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