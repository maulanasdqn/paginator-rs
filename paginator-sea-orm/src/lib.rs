use paginator_rs::{
    CursorDirection, CursorValue, FilterOperator, FilterValue, PaginationParams, PaginatorError,
    PaginatorResponse, PaginatorResponseMeta,
};
use sea_orm::{
    sea_query::{Alias, Condition, Expr, SimpleExpr},
    ConnectionTrait, EntityTrait, PaginatorTrait as SeaPaginatorTrait, QueryFilter, QuerySelect,
    Select,
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
    }
}

fn build_filter_condition(params: &PaginationParams) -> Condition {
    let mut condition = Condition::all();

    if let Some(ref cursor) = params.cursor {
        let col = Expr::col(Alias::new(&cursor.field));
        let cursor_val = cursor_value_to_sea_value(&cursor.value);

        let cursor_expr = match cursor.direction {
            CursorDirection::After => match params.sort_direction.as_ref() {
                Some(paginator_rs::SortDirection::Desc) => col.lt(cursor_val),
                _ => col.gt(cursor_val),
            },
            CursorDirection::Before => match params.sort_direction.as_ref() {
                Some(paginator_rs::SortDirection::Desc) => col.gt(cursor_val),
                _ => col.lt(cursor_val),
            },
        };

        condition = condition.add(cursor_expr);
    }

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
        let filter_condition = build_filter_condition(params);
        let mut query = self.filter(filter_condition.clone());

        let total = if params.disable_total_count {
            None
        } else {
            let count = query
                .clone()
                .count(db)
                .await
                .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;
            Some(count)
        };

        if params.cursor.is_some() {
            query = query.limit((params.limit() + 1) as u64);
        } else {
            query = query
                .offset(params.offset() as u64)
                .limit(params.limit() as u64);
        }

        let mut data = query
            .all(db)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?;

        let meta = if params.cursor.is_some() {
            let has_next = data.len() > params.per_page as usize;
            if has_next {
                data.truncate(params.per_page as usize);
            }
            PaginatorResponseMeta::new_with_cursors(
                params.page,
                params.per_page,
                total.map(|t| t as u32),
                has_next,
                None,
                None,
            )
        } else if let Some(count) = total {
            PaginatorResponseMeta::new(params.page, params.per_page, count as u32)
        } else {
            let has_next = data.len() as u32 > params.per_page;
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

    if let Some(ref field) = params.sort_by {
        if let Some(ref direction) = params.sort_direction {
            query = sort_fn(query, field, direction);
        }
    }

    query.paginate_with(db, params).await
}
