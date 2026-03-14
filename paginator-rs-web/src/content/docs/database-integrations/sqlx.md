---
title: SQLx
description: Paginate queries with SQLx for PostgreSQL, MySQL, and SQLite
---

The `paginator-sqlx` crate provides pagination support for SQLx with PostgreSQL, MySQL, and SQLite.

## Installation

```toml
[dependencies]
# PostgreSQL
paginator-sqlx = { version = "0.2.2", features = ["postgres", "runtime-tokio"] }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }

# MySQL
paginator-sqlx = { version = "0.2.2", features = ["mysql", "runtime-tokio"] }

# SQLite
paginator-sqlx = { version = "0.2.2", features = ["sqlite", "runtime-tokio"] }
```

## Basic Usage

```rust
use paginator_sqlx::postgres::paginate_query;
use paginator_rs::Paginator;
use sqlx::PgPool;

#[derive(sqlx::FromRow, serde::Serialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

async fn list_users(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let params = Paginator::new()
        .page(1)
        .per_page(10)
        .sort().desc("created_at")
        .build();

    let result = paginate_query::<_, User>(
        pool,
        "SELECT id, name, email FROM users WHERE active = true",
        &params,
    ).await?;

    println!("Page {}/{}", result.meta.page, result.meta.total_pages.unwrap());
    println!("Total: {}", result.meta.total.unwrap());

    Ok(())
}
```

## With Filters

```rust
let params = Paginator::new()
    .page(1)
    .per_page(20)
    .filter()
        .eq("status", "active")
        .gt("age", 18)
        .apply()
    .sort().desc("created_at")
    .build();

let result = paginate_query::<_, User>(
    pool,
    "SELECT * FROM users",
    &params,
).await?;
```

Filters are automatically converted to parameterized SQL WHERE clauses.

## With CTE (Common Table Expressions)

CTE queries work seamlessly:

```rust
let result = paginate_query::<_, Report>(
    pool,
    "WITH active_users AS (
        SELECT * FROM users WHERE active = true
    )
    SELECT * FROM active_users",
    &params,
).await?;
```

## MySQL and SQLite

The API is identical across databases — just change the import:

```rust
// MySQL
use paginator_sqlx::mysql::paginate_query;

// SQLite
use paginator_sqlx::sqlite::paginate_query;
```

## Field Name Validation

Use `validate_field_name()` to ensure sort/filter field names are safe:

```rust
use paginator_sqlx::validate_field_name;

// Returns true for valid SQL identifiers
assert!(validate_field_name("user_name"));
assert!(!validate_field_name("user; DROP TABLE"));
```
