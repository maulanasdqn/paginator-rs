use crate::common::{PaginateQuery, PaginatedQuery};
use crate::query_builder::QueryBuilderExt;
use paginator_rs::{
    CursorDirection, CursorValue, PaginationParams, PaginatorError, PaginatorResponse,
    PaginatorResponseMeta,
};
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

fn is_cte_query(query: &str) -> bool {
    query.trim().to_uppercase().starts_with("WITH")
}

pub async fn paginate_query<'e, E, T>(
    executor: E,
    base_query: &str,
    params: &PaginationParams,
) -> Result<PaginatorResponse<T>, PaginatorError>
where
    E: Executor<'e, Database = Postgres> + Clone,
    T: for<'r> FromRow<'r, PgRow> + Send + Unpin + Serialize,
{
    let has_filters_or_search = !params.filters.is_empty() || params.search.is_some();

    let count_query_str = if is_cte_query(base_query) {
        if has_filters_or_search {
            format!(
                "{}, _paginator_filtered AS (SELECT * FROM ({}) AS _base WHERE 1=1",
                base_query.trim_end_matches(';'),
                base_query
            )
        } else {
            format!("SELECT COUNT(*) FROM ({}) as count_subquery", base_query)
        }
    } else {
        if has_filters_or_search {
            format!("SELECT COUNT(*) FROM ({}) AS _base WHERE 1=1", base_query)
        } else {
            format!("SELECT COUNT(*) FROM ({}) as count_subquery", base_query)
        }
    };

    let total = if params.disable_total_count {
        None
    } else {
        let count = if has_filters_or_search {
            let mut count_builder: QueryBuilder<Postgres> = QueryBuilder::new(&count_query_str);
            count_builder.push_filters(params);
            count_builder.push_search(params);

            if is_cte_query(base_query) {
                count_builder.push(") SELECT COUNT(*) FROM _paginator_filtered");
            }

            let count_query = count_builder.build_query_as::<(i64,)>();
            let (total,) = count_query
                .fetch_one(executor.clone())
                .await
                .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;
            total
        } else {
            let (total,): (i64,) = sqlx::query_as(&count_query_str)
                .fetch_one(executor.clone())
                .await
                .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;
            total
        };
        Some(count)
    };

    let data_query_str = if is_cte_query(base_query) {
        if has_filters_or_search {
            format!(
                "{}, _paginator_filtered AS (SELECT * FROM ({}) AS _base WHERE 1=1",
                base_query.trim_end_matches(';'),
                base_query
            )
        } else {
            base_query.to_string()
        }
    } else {
        if has_filters_or_search {
            format!("SELECT * FROM ({}) AS _base WHERE 1=1", base_query)
        } else {
            base_query.to_string()
        }
    };

    let mut data_builder: QueryBuilder<Postgres> = QueryBuilder::new(&data_query_str);

    if has_filters_or_search {
        data_builder.push_filters(params);
        data_builder.push_search(params);

        if is_cte_query(base_query) {
            data_builder.push(") SELECT * FROM _paginator_filtered");
        }
    }

    if let Some(ref cursor) = params.cursor {
        let operator = match cursor.direction {
            CursorDirection::After => match params.sort_direction.as_ref() {
                Some(paginator_rs::SortDirection::Desc) => "<",
                _ => ">",
            },
            CursorDirection::Before => match params.sort_direction.as_ref() {
                Some(paginator_rs::SortDirection::Desc) => ">",
                _ => "<",
            },
        };

        if !has_filters_or_search {
            data_builder.push(" WHERE ");
        } else {
            data_builder.push(" AND ");
        }

        data_builder.push(&cursor.field);
        data_builder.push(" ");
        data_builder.push(operator);
        data_builder.push(" ");

        match &cursor.value {
            CursorValue::String(s) => {
                data_builder.push_bind(s.clone());
            }
            CursorValue::Int(i) => {
                data_builder.push_bind(*i);
            }
            CursorValue::Float(f) => {
                data_builder.push_bind(*f);
            }
        }
    }

    if let Some(ref sort_field) = params.sort_by {
        data_builder.push(" ORDER BY ");
        data_builder.push(sort_field);
        match params.sort_direction.as_ref() {
            Some(paginator_rs::SortDirection::Desc) => data_builder.push(" DESC"),
            _ => data_builder.push(" ASC"),
        };
    }

    if params.cursor.is_some() {
        data_builder.push(" LIMIT ");
        data_builder.push_bind((params.limit() + 1) as i64);
    } else {
        data_builder.push(" LIMIT ");
        data_builder.push_bind(params.limit() as i64);
        data_builder.push(" OFFSET ");
        data_builder.push_bind(params.offset() as i64);
    }

    let data_query = data_builder.build_query_as::<T>();
    let mut data = data_query
        .fetch_all(executor)
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
