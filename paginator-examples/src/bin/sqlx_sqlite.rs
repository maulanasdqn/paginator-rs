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

    println!("\n=== Cursor pagination ===");
    // Serve page 1 with offset pagination and hand out a cursor for page 2.
    let params = PaginatorBuilder::new().per_page(2).sort_by("id").build();
    let first = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params)
        .await?
        .with_cursors("id");
    println!(
        "page 1: {:?} next_cursor={}",
        first.data.iter().map(|u| u.id).collect::<Vec<_>>(),
        first.meta.next_cursor.as_deref().unwrap_or("-")
    );

    // Follow next_cursor to the end, skipping COUNT(*) on every hop.
    let mut next = first.meta.next_cursor.clone();
    let mut last_page = None;
    while let Some(cursor) = next {
        let params = PaginatorBuilder::new()
            .per_page(2)
            .sort_by("id")
            .cursor_from_encoded(&cursor)?
            .disable_total_count()
            .build();
        let page = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params).await?;
        println!(
            "after cursor: {:?} has_next={} has_prev={}",
            page.data.iter().map(|u| u.id).collect::<Vec<_>>(),
            page.meta.has_next,
            page.meta.has_prev
        );
        next = page.meta.next_cursor.clone();
        last_page = Some(page);
    }

    // Walk one page back from the last page with prev_cursor.
    if let Some(prev) = last_page.and_then(|p| p.meta.prev_cursor) {
        let params = PaginatorBuilder::new()
            .per_page(2)
            .sort_by("id")
            .cursor_from_encoded(&prev)?
            .build();
        let page = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params).await?;
        println!(
            "before cursor: {:?}",
            page.data.iter().map(|u| u.id).collect::<Vec<_>>()
        );
    }

    println!("\n=== Relative cursor pagination ===");
    // `page` is an offset in pages relative to the cursor: the 2nd page after id 2.
    let params = PaginatorBuilder::new()
        .per_page(2)
        .page(2)
        .cursor_after("id", CursorValue::Int(2))
        .build();
    let result = paginate_query::<_, User>(&pool, "SELECT * FROM users", &params).await?;
    println!(
        "page 2 after id=2: {:?} has_next={}",
        result.data.iter().map(|u| u.id).collect::<Vec<_>>(),
        result.meta.has_next
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
