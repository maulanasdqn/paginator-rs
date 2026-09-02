//! All 14 filter operators, every `FilterValue` type, and SQL generation.
//!
//! Run with: cargo run -p paginator-examples --bin filters

use paginator_rs::{Filter, FilterOperator, FilterValue, PaginatorBuilder};

fn main() {
    println!("=== Every FilterValue type ===");
    let values = vec![
        FilterValue::String("active".into()),
        FilterValue::Int(42),
        FilterValue::Float(9.99),
        FilterValue::Bool(true),
        FilterValue::Null,
        FilterValue::Array(vec![FilterValue::Int(1), FilterValue::Int(2)]),
    ];
    for v in &values {
        println!("{:?} -> SQL literal: {}", v, v.to_sql_string());
    }

    println!("\n=== Every FilterOperator as a WHERE clause ===");
    let filters = vec![
        Filter::new(
            "status",
            FilterOperator::Eq,
            FilterValue::String("active".into()),
        ),
        Filter::new(
            "role",
            FilterOperator::Ne,
            FilterValue::String("bot".into()),
        ),
        Filter::new("age", FilterOperator::Gt, FilterValue::Int(18)),
        Filter::new("age", FilterOperator::Lt, FilterValue::Int(65)),
        Filter::new("score", FilterOperator::Gte, FilterValue::Float(4.5)),
        Filter::new("retries", FilterOperator::Lte, FilterValue::Int(3)),
        Filter::new(
            "name",
            FilterOperator::Like,
            FilterValue::String("Jo%".into()),
        ),
        Filter::new(
            "email",
            FilterOperator::ILike,
            FilterValue::String("%@example.com".into()),
        ),
        Filter::new(
            "role",
            FilterOperator::In,
            FilterValue::Array(vec![
                FilterValue::String("admin".into()),
                FilterValue::String("moderator".into()),
            ]),
        ),
        Filter::new(
            "role",
            FilterOperator::NotIn,
            FilterValue::Array(vec![FilterValue::String("bot".into())]),
        ),
        Filter::new(
            "created_at",
            FilterOperator::Between,
            FilterValue::Array(vec![FilterValue::Int(2020), FilterValue::Int(2024)]),
        ),
        Filter::new("deleted_at", FilterOperator::IsNull, FilterValue::Null),
        Filter::new("email", FilterOperator::IsNotNull, FilterValue::Null),
        Filter::new(
            "bio",
            FilterOperator::Contains,
            FilterValue::String("rust".into()),
        ),
    ];
    for f in &filters {
        println!("{:<12?} -> {}", f.operator, f.to_sql_where());
    }

    println!("\n=== SurrealQL variant ===");
    for f in filters.iter().take(3) {
        println!("{:<12?} -> {}", f.operator, f.to_surrealql_where());
    }

    println!("\n=== Combined WHERE clause from params ===");
    let params = PaginatorBuilder::new()
        .filter_eq("status", FilterValue::String("active".into()))
        .filter_gt("age", FilterValue::Int(18))
        .search("developer", vec!["title".into(), "bio".into()])
        .build();
    println!("SQL:       WHERE {}", params.to_sql_where().unwrap());
    println!("SurrealQL: WHERE {}", params.to_surrealql_where().unwrap());
}
