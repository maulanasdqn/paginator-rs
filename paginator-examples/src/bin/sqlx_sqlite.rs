//! SQLx integration with an in-memory SQLite database. No external services needed.
//!
//! Run with: cargo run -p paginator-examples --bin sqlx_sqlite

use paginator_rs::{CursorValue, FilterValue, PaginatorBuilder};
use paginator_sqlx::sqlite::paginate_query;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow, serde::Serialize, Debug)]
struct User {
    id: i64,
    name: String,
    email: String,
    age: i64,
    active: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    sqlx::query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            age INTEGER NOT NULL,
            active BOOLEAN NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    for (name, email, age, active) in [
        ("John Doe", "john@doe.com", 28, true),
        ("Jane Doe", "jane@doe.com", 34, true),
        ("Bob Smith", "bob@smith.com", 17, true),
        ("Alice Brown", "alice@brown.com", 45, false),
        ("Charlie Davis", "charlie@davis.com", 52, true),
        ("Diana Evans", "diana@evans.com", 23, true),
    ] {
        sqlx::query("INSERT INTO users (name, email, age, active) VALUES (?, ?, ?, ?)")
            .bind(name)
            .bind(email)
            .bind(age)
            .bind(active)
            .execute(&pool)
            .await?;
    }

    println!("=== Basic pagination ===");
    let params = PaginatorBuilder::new().page(1).per_page(3).build();
    let result = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params).await?;
    println!(
        "page {}/{} total={:?}: {:?}",
        result.meta.page,
        result.meta.total_pages.unwrap(),
        result.meta.total,
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Sorting ===");
    let params = PaginatorBuilder::new()
        .per_page(3)
        .sort_by("age")
        .sort_desc()
        .build();
    let result = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params).await?;
    for u in &result.data {
        println!("{} ({})", u.name, u.age);
    }

    println!("\n=== Filters (parameterized, injection-safe) ===");
    let params = PaginatorBuilder::new()
        .filter_eq("active", FilterValue::Bool(true))
        .filter_gt("age", FilterValue::Int(18))
        .build();
    let result = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params).await?;
    println!(
        "adults, active: {:?}",
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Search across fields ===");
    let params = PaginatorBuilder::new()
        .search("doe", vec!["name".into(), "email".into()])
        .build();
    let result = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params).await?;
    println!(
        "matching 'doe': {:?}",
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Cursor pagination without COUNT(*) ===");
    let params = PaginatorBuilder::new()
        .per_page(2)
        .sort_by("id")
        .cursor_after("id", CursorValue::Int(2))
        .disable_total_count()
        .build();
    let result = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params).await?;
    println!(
        "after id=2: {:?} has_next={} total={:?}",
        result.data.iter().map(|u| u.id).collect::<Vec<_>>(),
        result.meta.has_next,
        result.meta.total
    );

    println!("\n=== CTE (WITH clause) query ===");
    let params = PaginatorBuilder::new().per_page(10).build();
    let result = paginate_query::<_, User>(
        &pool,
        "WITH adults AS (SELECT * FROM users WHERE age >= 18) SELECT * FROM adults",
        &params,
    )
    .await?;
    println!("adults via CTE: {}", result.data.len());

    Ok(())
}
