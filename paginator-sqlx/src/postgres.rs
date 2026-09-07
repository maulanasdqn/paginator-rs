use crate::common::{
    clean_base_query, count_sql, data_sql, finish, limit_offset, order_for, PaginateQuery,
    PaginatedQuery,
};
use crate::query_builder::QueryBuilderExt;
use paginator_rs::{PaginationParams, PaginatorError, PaginatorResponse};
use paginator_utils::IntoPaginationParams;
use serde::Serialize;
use sqlx::postgres::{PgArguments, PgRow};
use sqlx::query_builder::QueryBuilder;
use sqlx::{query::Query, Executor, FromRow, Postgres};

impl<'q, T> PaginateQuery<'q, Postgres, T> for Query<'q, Postgres, PgArguments>
where
    T: Send + Unpin,
{
    fn paginate(self, params: &PaginationParams) -> PaginatedQuery<'q, Postgres, T> {
        PaginatedQuery::new("", params)
    }
}

/// Paginate `base_query`, applying the filters, search, sort, and cursor in `params`.
///
/// Filters, search, and the keyset predicate are appended by wrapping the base
/// query in a derived table, so the base query may carry its own `WHERE` clause
/// or start with a common table expression. Without them (and without a cursor)
/// the base query is used as is and `ORDER BY`/`LIMIT` are appended directly.
///
/// With a cursor the rows are ordered by the cursor field, `per_page + 1` rows
/// are fetched to detect the next page, and `next_cursor`/`prev_cursor` are
/// derived from the boundary rows. `page` then acts as an offset in pages
/// relative to the cursor.
pub async fn paginate_query<'e, E, T, P>(
    executor: E,
    base_query: &str,
    params: P,
) -> Result<PaginatorResponse<T>, PaginatorError>
where
    E: Executor<'e, Database = Postgres> + Clone,
    T: for<'r> FromRow<'r, PgRow> + Send + Unpin + Serialize,
    P: IntoPaginationParams,
{
    let params = params.into_pagination_params();
    let plan = params.keyset_plan().map_err(PaginatorError::Custom)?;
    let has_filters_or_search = !params.filters.is_empty() || params.search.is_some();
    let base_query = clean_base_query(base_query);

    let total = if params.disable_total_count {
        None
    } else {
        let mut count_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(count_sql(base_query, has_filters_or_search));
        if has_filters_or_search {
            count_builder.push_filters(&params);
            count_builder.push_search(&params);
        }
        let (total,): (i64,) = count_builder
            .build_query_as()
            .fetch_one(executor.clone())
            .await
            .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;
        Some(total)
    };

    let mut data_builder: QueryBuilder<Postgres> = QueryBuilder::new(data_sql(
        base_query,
        has_filters_or_search || plan.is_some(),
    ));
    if has_filters_or_search {
        data_builder.push_filters(&params);
        data_builder.push_search(&params);
    }
    if let Some(ref plan) = plan {
        data_builder.push(" AND ");
        data_builder.push_keyset(plan, Some("::uuid"));
    }
    if let Some((field, direction)) = order_for(&params, plan.as_ref())? {
        data_builder.push_order_by(field, direction);
    }
    let (limit, offset) = limit_offset(&params, plan.is_some());
    data_builder.push(" LIMIT ");
    data_builder.push_bind(limit);
    data_builder.push(" OFFSET ");
    data_builder.push_bind(offset);

    let data = data_builder
        .build_query_as::<T>()
        .fetch_all(executor)
        .await
        .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?;

    Ok(finish(data, &params, plan.as_ref(), total))
}
