---
title: Quick Start
description: Get up and running with paginator-rs in minutes
---

## Basic Usage

Define your data model and create pagination parameters using the builder pattern:

```rust
use paginator_rs::{Paginator, PaginationParams, PaginatorBuilder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Using the modern fluent builder
let params = Paginator::new()
    .page(1)
    .per_page(20)
    .sort().asc("name")
    .build();

// Or using the legacy builder
let params = PaginatorBuilder::new()
    .page(1)
    .per_page(20)
    .sort_by("name")
    .sort_asc()
    .build();

// Or create directly
let params = PaginationParams::new(1, 20);
```

## With a Database

Here's a complete example using SQLx with PostgreSQL:

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
    println!("Total users: {}", result.meta.total.unwrap());

    for user in &result.data {
        println!("  {} - {}", user.name, user.email);
    }

    Ok(())
}
```

## With a Web Framework

Here's a complete example using Axum:

```rust
use axum::{Router, routing::get};
use paginator_axum::{PaginationQuery, PaginatedJson};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
}

async fn get_users(
    PaginationQuery(params): PaginationQuery,
) -> PaginatedJson<User> {
    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    // Automatically adds pagination headers
    PaginatedJson::new(users, &params, 100)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/users", get(get_users));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

Then query with:

```
GET /users?page=2&per_page=20&sort_by=name&sort_direction=asc
```
