//! SurrealDB integration with the in-memory engine. No external services needed.
//!
//! Run with: cargo run -p paginator-examples --bin surrealdb_mem

use paginator_rs::{FilterValue, PaginatorBuilder};
use paginator_surrealdb::{paginate_query, paginate_table, QueryBuilder};
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct User {
    name: String,
    email: String,
    age: i64,
    active: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("example").use_db("example").await?;

    for (name, email, age, active) in [
        ("John Doe", "john@doe.com", 28, true),
        ("Jane Doe", "jane@doe.com", 34, true),
        ("Bob Smith", "bob@smith.com", 17, true),
        ("Alice Brown", "alice@brown.com", 45, false),
        ("Charlie Davis", "charlie@davis.com", 52, true),
    ] {
        let _: Option<User> = db
            .create("users")
            .content(User {
                name: name.to_string(),
                email: email.to_string(),
                age,
                active,
            })
            .await?;
    }

    println!("=== Raw query ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(2)
        .sort_by("name")
        .sort_asc()
        .build();
    let result =
        paginate_query::<User, _>(&db, "SELECT * FROM users WHERE active = true", &params).await?;
    println!(
        "page {}/{}: {:?}",
        result.meta.page,
        result.meta.total_pages.unwrap(),
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Table helper ===");
    let params = PaginatorBuilder::new().per_page(10).build();
    let result = paginate_table::<User, _>(&db, "users", Some("age > 18"), &params).await?;
    println!(
        "adults: {:?}",
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Query builder ===");
    let params = PaginatorBuilder::new().per_page(10).build();
    let result = QueryBuilder::new()
        .select("*")
        .from("users")
        .where_clause("active = true")
        .and("age > 18")
        .paginate::<User, _>(&db, &params)
        .await?;
    println!(
        "active adults: {:?}",
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Paginator filters and search ===");
    let params = PaginatorBuilder::new()
        .filter_gt("age", FilterValue::Int(30))
        .build();
    let result = paginate_query::<User, _>(&db, "SELECT * FROM users", &params).await?;
    println!(
        "age > 30: {:?}",
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Without COUNT ===");
    let params = PaginatorBuilder::new()
        .per_page(2)
        .disable_total_count()
        .build();
    let result = paginate_query::<User, _>(&db, "SELECT * FROM users", &params).await?;
    println!(
        "{} rows, has_next={} total={:?}",
        result.data.len(),
        result.meta.has_next,
        result.meta.total
    );

    Ok(())
}
