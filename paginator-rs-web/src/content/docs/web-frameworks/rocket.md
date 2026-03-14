---
title: Rocket
description: Request guards and responders for Rocket
---

The `paginator-rocket` crate provides request guards and responders for Rocket.

## Installation

```toml
[dependencies]
paginator-rocket = "0.2.2"
rocket = { version = "0.5", features = ["json"] }
```

## Pagination Request Guard

The `Pagination` guard automatically extracts pagination parameters from the request URI:

```rust
use rocket::{get, routes};
use paginator_rocket::{Pagination, PaginatedJson};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
}

#[get("/users")]
async fn get_users(pagination: Pagination) -> PaginatedJson<User> {
    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    PaginatedJson::new(users, &pagination.params, 100)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/api", routes![get_users])
}
```

## Query Parameters

The guard parses from the URI query string:

```
GET /api/users?page=2&per_page=20&sort_by=name&sort_direction=asc
```

| Parameter | Type | Default |
|-----------|------|---------|
| `page` | `u32` | `1` |
| `per_page` | `u32` | `20` |
| `sort_by` | `String` | - |
| `sort_direction` | `String` | - |

## PaginatedJson Responder

`PaginatedJson` serializes the response and adds pagination headers:

```rust
PaginatedJson::new(data, &params, total_count)
```

## Helper Function

Use `create_paginated_response` for custom response building:

```rust
use paginator_rocket::create_paginated_response;

let response = create_paginated_response(users, &params, 100);
```
