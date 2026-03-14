---
title: SurrealDB
description: Pagination support for SurrealDB multi-model database
---

The `paginator-surrealdb` crate provides pagination support for SurrealDB.

## Installation

```toml
[dependencies]
paginator-surrealdb = { version = "0.2.2", features = ["protocol-ws", "kv-mem"] }
surrealdb = { version = "2.1", features = ["protocol-ws", "kv-mem"] }
```

## Raw Query

```rust
use paginator_surrealdb::paginate_query;
use paginator_rs::Paginator;

#[derive(serde::Deserialize, serde::Serialize)]
struct User {
    id: String,
    name: String,
    email: String,
    active: bool,
}

let params = Paginator::new()
    .page(1)
    .per_page(10)
    .sort().asc("name")
    .build();

let result = paginate_query::<User, _>(
    &db,
    "SELECT * FROM users WHERE active = true",
    &params,
).await?;
```

## Table Helper

For simple table queries:

```rust
use paginator_surrealdb::paginate_table;

let result = paginate_table::<User, _>(
    &db,
    "users",
    Some("active = true"),
    &params,
).await?;
```

## Query Builder

The fluent query builder provides a more ergonomic API:

```rust
use paginator_surrealdb::QueryBuilder;

let result = QueryBuilder::new()
    .select("*")
    .from("users")
    .where_clause("active = true")
    .and("age > 18")
    .paginate::<User, _>(&db, &params)
    .await?;
```

## SurrealQL WHERE Clauses

Filters are automatically converted to SurrealQL WHERE clauses using `to_surrealql_where()`:

```rust
let params = PaginatorBuilder::new()
    .filter_eq("status", FilterValue::String("active".to_string()))
    .filter_gt("age", FilterValue::Int(18))
    .build();

if let Some(where_clause) = params.to_surrealql_where() {
    println!("{}", where_clause);
    // Output: status = 'active' AND age > 18
}
```
