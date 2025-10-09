//! SeaORM integration for paginator-rs
//!
//! This crate provides pagination support for SeaORM entities with type-safe query building.
//!
//! # Example
//!
//! ```ignore
//! use sea_orm::{Database, EntityTrait};
//! use paginator_sea_orm::PaginateSeaOrm;
//! use paginator_rs::PaginatorBuilder;
//!
//! // Assuming you have a User entity
//! async fn get_users(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
//!     let params = PaginatorBuilder::new()
//!         .page(1)
//!         .per_page(10)
//!         .build();
//!
//!     let result = User::find()
//!         .paginate_with(db, &params)
//!         .await?;
//!
//!     println!("Users: {:?}", result);
//!     Ok(())
//! }
//! ```

use paginator_rs::{
    FilterOperator, FilterValue, PaginationParams, PaginatorError, PaginatorResponse,
    PaginatorResponseMeta,
};
use sea_orm::{
    sea_query::{Alias, Condition, Expr, SimpleExpr},
    ConnectionTrait, EntityTrait, PaginatorTrait as SeaPaginatorTrait, QueryFilter, QuerySelect,
    Select,
};
use serde::Serialize;

/// Convert a FilterValue to a sea_query Value
fn filter_value_to_sea_value(value: &FilterValue) -> sea_orm::sea_query::Value {
    match value {
        FilterValue::String(s) => s.clone().into(),
        FilterValue::Int(i) => (*i).into(),
        FilterValue::Float(f) => (*f).into(),
        FilterValue::Bool(b) => (*b).into(),
        FilterValue::Null => sea_orm::sea_query::Value::String(None),
        FilterValue::Array(_) => {
            // Arrays are handled separately for IN/NOT IN operators
            sea_orm::sea_query::Value::String(None)
        }
    }
}

/// Build a SeaORM condition from our filter system
fn build_filter_condition(params: &PaginationParams) -> Condition {
    let mut condition = Condition::all();

    // Apply filters
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
                // SeaORM doesn't have native ILIKE, use LIKE with LOWER
                Expr::expr(Expr::cust(&format!("LOWER({})", filter.field)))
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
            _ => continue, // Skip unsupported operator/value combinations
        };

        condition = condition.add(filter_expr);
    }

    // Apply search
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
                // Use LOWER for case-insensitive search
                Expr::expr(Expr::cust(&format!("LOWER({})", field))).like(pattern.to_lowercase())
            };

            search_condition = search_condition.add(search_expr);
        }

        condition = condition.add(search_condition);
    }

    condition
}

/// Extension trait for paginating SeaORM queries
#[async_trait::async_trait]
pub trait PaginateSeaOrm<'db, C>
where
    C: ConnectionTrait,
{
    type Item;

    /// Paginate this query with the given parameters
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
        // Apply filters and search conditions
        let filter_condition = build_filter_condition(params);
        let mut query = self.filter(filter_condition.clone());

        // Get total count (with filters applied)
        let total = query
            .clone()
            .count(db)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Count query failed: {}", e)))?;

        // Apply offset and limit
        query = query
            .offset(params.offset() as u64)
            .limit(params.limit() as u64);

        // Execute query
        let data = query
            .all(db)
            .await
            .map_err(|e| PaginatorError::Custom(format!("Paginated query failed: {}", e)))?;

        Ok(PaginatorResponse {
            data,
            meta: PaginatorResponseMeta::new(params.page, params.per_page, total as u32),
        })
    }
}

/// Helper function to paginate any SeaORM Select query
///
/// # Example
///
/// ```ignore
/// use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
/// use paginator_sea_orm::paginate;
/// use paginator_rs::PaginationParams;
///
/// async fn get_active_users(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
///     let params = PaginationParams::new(1, 20);
///
///     let query = User::find()
///         .filter(user::Column::Active.eq(true));
///
///     let result = paginate(query, db, &params).await?;
///     Ok(())
/// }
/// ```
pub async fn paginate<'db, C, E>(
    select: Select<E>,
    db: &'db C,
    params: &PaginationParams,
) -> Result<PaginatorResponse<<E as EntityTrait>::Model>, PaginatorError>
where
    C: ConnectionTrait,
    E: EntityTrait,
    <E as EntityTrait>::Model: Serialize + Send + Sync,
{
    select.paginate_with(db, params).await
}

/// Paginate with sorting support
///
/// # Example
///
/// ```ignore
/// use sea_orm::Order;
/// use paginator_sea_orm::paginate_with_sort;
/// use paginator_rs::{PaginationParams, SortDirection};
///
/// async fn get_sorted_users(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
///     let params = PaginationParams::new(1, 20)
///         .with_sort("name")
///         .with_direction(SortDirection::Asc);
///
///     let query = User::find();
///
///     let result = paginate_with_sort(
///         query,
///         db,
///         &params,
///         |q, field, direction| {
///             match field {
///                 "name" => {
///                     if direction == &SortDirection::Asc {
///                         q.order_by_asc(user::Column::Name)
///                     } else {
///                         q.order_by_desc(user::Column::Name)
///                     }
///                 },
///                 _ => q,
///             }
///         }
///     ).await?;
///     Ok(())
/// }
/// ```
pub async fn paginate_with_sort<'db, C, E, F>(
    select: Select<E>,
    db: &'db C,
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

    // Apply sorting if specified
    if let Some(ref field) = params.sort_by {
        if let Some(ref direction) = params.sort_direction {
            query = sort_fn(query, field, direction);
        }
    }

    query.paginate_with(db, params).await
}
