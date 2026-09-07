use paginator_rs::{
    CursorValue, FilterOperator, FilterValue, KeysetPlan, PaginationParams, PaginatorError,
    PaginatorResponse, PaginatorResponseMeta, SortDirection,
};
use sea_orm::{
    sea_query::{Alias, Condition, Expr, ExprTrait, SimpleExpr},
    ConnectionTrait, EntityTrait, Order, PaginatorTrait as SeaPaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Select,
};
use serde::Serialize;

fn filter_value_to_sea_value(value: &FilterValue) -> sea_orm::sea_query::Value {
    match value {
        FilterValue::String(s) => s.clone().into(),
        FilterValue::Int(i) => (*i).into(),
        FilterValue::Float(f) => (*f).into(),
        FilterValue::Bool(b) => (*b).into(),
        FilterValue::Null => sea_orm::sea_query::Value::String(None),
        FilterValue::Array(_) => sea_orm::sea_query::Value::String(None),
    }
}

fn cursor_value_to_sea_value(value: &CursorValue) -> sea_orm::sea_query::Value {
    match value {
        CursorValue::String(s) => s.clone().into(),
        CursorValue::Int(i) => (*i).into(),
        CursorValue::Float(f) => (*f).into(),
        // Parse UUID string and convert to sea-orm UUID value
        CursorValue::Uuid(u) => {
            if let Ok(parsed) = uuid::Uuid::parse_str(u) {
                sea_orm::sea_query::Value::Uuid(Some(parsed))
            } else {
                // Fallback to string if parsing fails
                u.clone().into()
            }
        }
    }
}

/// The keyset predicate selecting rows on the far side of the cursor in query order.
fn keyset_condition(plan: &KeysetPlan<'_>) -> SimpleExpr {
    let col = Expr::col(Alias::new(plan.field()));
    let value = cursor_value_to_sea_value(plan.value());
    match plan.query_sort {
        SortDirection::Asc => col.gt(value),
        SortDirection::Desc => col.lt(value),
    }
}

fn sea_order(direction: SortDirection) -> Order {
    match direction {
        SortDirection::Asc => Order::Asc,
        SortDirection::Desc => Order::Desc,
    }
}

/// Filters and search from `params`. The cursor is applied separately so the
/// total count reflects the whole result set, not just the rows past the cursor.
fn build_filter_condition(params: &PaginationParams) -> Condition {
    let mut condition = Condition::all();

    for filter in &params.filters {
        let col = Expr::col(Alias::new(&filter.field));

        let filter_expr: SimpleExpr = match (&filter.operator, &filter.value) {
            (FilterOperator::Eq, value) => col.eq(filter_value_to_sea_value(value)),
            (FilterOperator::Ne, value) => col.ne(filter_value_to_sea_value(value)),
            (FilterOperator::Gt, value) => col.gt(filter_value_to_sea_value(value)),
            (FilterOperator::Lt, value) => col.lt(filter_value_to_sea_value(value)),
            (FilterOperator::Gte, value) => col.gte(filter_value_to_sea_value(value)),
            (FilterOperator::Lte, value) => col.lte(filter_value_to_sea_value(value)),
            (FilterOperator::Like, FilterValue::String(pattern)) => col.like(pattern.clone()),
            (FilterOperator::ILike, FilterValue::String(pattern)) => {
                Expr::expr(Expr::cust(format!("LOWER({})", filter.field)))
                    .like(pattern.to_lowercase())
            }
            (FilterOperator::In, FilterValue::Array(values)) => {
                let sea_values: Vec<sea_orm::sea_query::Value> =
                    values.iter().map(filter_value_to_sea_value).collect();
                col.is_in(sea_values)
            }
            (FilterOperator::NotIn, FilterValue::Array(values)) => {
                let sea_values: Vec<sea_orm::sea_query::Value> =
                    values.iter().map(filter_value_to_sea_value).collect();
                col.is_not_in(sea_values)
            }
            (FilterOperator::IsNull, _) => col.is_null(),
            (FilterOperator::IsNotNull, _) => col.is_not_null(),
            (FilterOperator::Between, FilterValue::Array(values)) if values.len() == 2 => col
                .between(
                    filter_value_to_sea_value(&values[0]),
                    filter_value_to_sea_value(&values[1]),
                ),
            (FilterOperator::Contains, FilterValue::String(value)) => {
                col.like(format!("%{}%", value))
            }
            _ => continue,
        };

        condition = condition.add(filter_expr);
    }

    if let Some(ref search) = params.search {
        let mut search_condition = Condition::any();

        for field in &search.fields {
            let col = Expr::col(Alias::new(field));
            let pattern = if search.exact_match {
                search.query.clone()
            } else {
                format!("%{}%", search.query)
            };

            let search_expr = if search.case_sensitive {
                col.like(pattern)
            } else {
                Expr::expr(Expr::cust(format!("LOWER({})", field))).like(pattern.to_lowercase())
            };

            search_condition = search_condition.add(search_expr);
        }

        condition = condition.add(search_condition);
    }

    condition
}

#[async_trait::async_trait]
pub trait PaginateSeaOrm<'db, C>
where
    C: ConnectionTrait,
{
    type Item;

    /// Paginate this select with `params`.
    ///
    /// Filters and search are applied to both the count and the data query. With a
    /// cursor, rows are ordered by the cursor field (`sort_direction`, ascending by
    /// default), `per_page + 1` rows are fetched to detect the next page, and
    /// `next_cursor`/`prev_cursor` are derived from the boundary rows. `page` then
    /// acts as an offset in pages relative to the cursor. Without a cursor,
    /// `sort_by` is not applied here; use [`paginate_with_sort`] to map it to an
    /// entity column.
    async fn paginate_with(
        self,
        db: &'db C,
        params: &PaginationParams,
    ) -> Result<PaginatorResponse<Self::Item>, PaginatorError>;
}

#[async_trait::async_trait]
impl<'db, C, E> PaginateSeaOrm<'db, C> for Select<E>
where
    C: ConnectionTrait,
    E: EntityTrait,
    <E as EntityTrait>::Model: Serialize + Send + Sync,
{
    type Item = <E as EntityTrait>::Model;

    async fn paginate_with(
        self,
        db: &'db C,
        params: &PaginationParams,
    ) -> Result<PaginatorResponse<Self::Item>, PaginatorError> {
        let plan = params.keyset_plan().map_err(PaginatorError::Custom)?;
        let mut query = self.filter(build_filter_condition(params));

        let total = if params.disable_total_count {
            None
        } else {
            let count = query
                .clone()
                .count(db)
                .await
                .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;
            Some(count as u32)
        };

        if let Some(ref plan) = plan {
            query = query
                .filter(Condition::all().add(keyset_condition(plan)))
                .order_by(
                    Expr::col(Alias::new(plan.field())),
                    sea_order(plan.query_sort),
                );
        }

        // One extra row detects the next page when there is no total to derive
        // it from: in cursor mode and when the count query is disabled.
        let probe = plan.is_some() || params.disable_total_count;
        let limit = params.limit() as u64 + u64::from(probe);
        query = query.offset(params.offset() as u64).limit(limit);

        let mut data = query
            .all(db)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?;

        let meta = if let Some(ref plan) = plan {
            PaginatorResponseMeta::from_cursor_page(&mut data, params, plan, total)
        } else if let Some(count) = total {
            PaginatorResponseMeta::new(params.page, params.per_page, count)
        } else {
            let has_next = data.len() > params.per_page as usize;
            data.truncate(params.per_page as usize);
            PaginatorResponseMeta::new_without_total(params.page, params.per_page, has_next)
        };

        Ok(PaginatorResponse { data, meta })
    }
}

pub async fn paginate<C, E>(
    select: Select<E>,
    db: &C,
    params: &PaginationParams,
) -> Result<PaginatorResponse<<E as EntityTrait>::Model>, PaginatorError>
where
    C: ConnectionTrait,
    E: EntityTrait,
    <E as EntityTrait>::Model: Serialize + Send + Sync,
{
    select.paginate_with(db, params).await
}

/// Like [`paginate`], mapping `sort_by`/`sort_direction` to an `ORDER BY` through
/// `sort_fn`. When `params` carries a cursor the ordering is fixed by the cursor
/// field and `sort_fn` is not called.
pub async fn paginate_with_sort<C, E, F>(
    select: Select<E>,
    db: &C,
    params: &PaginationParams,
    sort_fn: F,
) -> Result<PaginatorResponse<<E as EntityTrait>::Model>, PaginatorError>
where
    C: ConnectionTrait,
    E: EntityTrait,
    <E as EntityTrait>::Model: Serialize + Send + Sync,
    F: FnOnce(Select<E>, &str, &paginator_rs::SortDirection) -> Select<E>,
{
    let mut query = select;

    if params.cursor.is_none() {
        if let Some(ref field) = params.sort_by {
            if let Some(ref direction) = params.sort_direction {
                query = sort_fn(query, field, direction);
            }
        }
    }

    query.paginate_with(db, params).await
}
