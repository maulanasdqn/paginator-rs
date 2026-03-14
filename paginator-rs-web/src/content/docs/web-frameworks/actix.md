---
title: Actix-web
description: Extractors, responders, and middleware for Actix-web
---

The `paginator-actix` crate provides extractors, responders, and optional middleware for Actix-web.

## Installation

```toml
[dependencies]
paginator-actix = "0.2.2"
actix-web = "4"
```

## PaginationQuery Extractor

```rust
use actix_web::{get, web, App, HttpServer};
use paginator_actix::{PaginationQuery, PaginatedJson};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
}

#[get("/users")]
async fn get_users(
    query: web::Query<PaginationQuery>,
) -> PaginatedJson<User> {
    let params = query.as_params();
    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    PaginatedJson::new(users, &params, 100)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(get_users)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Converting to PaginationParams

The `PaginationQuery` struct provides two methods:

```rust
// Borrow params
let params = query.as_params();

// Consume and convert
let params = query.into_inner().into_params();
```

## Pagination Middleware

Optional middleware that processes pagination parameters:

```rust
use paginator_actix::middleware::PaginationMiddleware;

App::new()
    .wrap(PaginationMiddleware)
    .service(get_users)
```

## PaginatedJson Responder

Automatically serializes data and adds pagination headers:

```rust
PaginatedJson::new(data, &params, total_count)
```

### Response Headers

```
X-Total-Count: 100
X-Total-Pages: 5
X-Current-Page: 1
X-Per-Page: 20
```

## Helper Function

```rust
use paginator_actix::create_paginated_response;

let response = create_paginated_response(users, &params, 100);
```
