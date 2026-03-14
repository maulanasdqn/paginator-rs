---
title: Axum
description: Pagination extractors and responders for Axum
---

The `paginator-axum` crate provides query extractors and JSON responders for Axum.

## Installation

```toml
[dependencies]
paginator-axum = "0.2.2"
axum = "0.7"
```

## PaginationQuery Extractor

Extract pagination parameters from query strings:

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

    PaginatedJson::new(users, &params, 100)
}

let app = Router::new().route("/users", get(get_users));
```

## Query Parameters

The extractor parses these query parameters:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | `u32` | `1` | Page number (1-indexed) |
| `per_page` | `u32` | `20` | Items per page (max: 100) |
| `sort_by` | `String` | - | Field to sort by |
| `sort_direction` | `String` | - | `asc` or `desc` |
| `filter` | `String[]` | - | Filters in `field:operator:value` format |
| `search` | `String` | - | Search query |
| `search_fields` | `String` | - | Comma-separated fields to search |

### Example Requests

```
GET /users?page=2&per_page=20
GET /users?sort_by=name&sort_direction=asc
GET /users?filter=status:eq:active&filter=age:gt:18
GET /users?search=john&search_fields=name,email
GET /users?filter=role:in:admin,moderator&sort_by=created_at&sort_direction=desc
```

### Filter Format

Filters use the format `field:operator:value`:

```
status:eq:active        # Equal
age:gt:18               # Greater than
age:between:18,65       # Between
role:in:admin,mod       # In array
name:like:%john%        # LIKE pattern
deleted_at:is_null      # IS NULL
```

## PaginatedJson Responder

`PaginatedJson` automatically serializes the response and adds pagination headers:

```rust
// With total count
PaginatedJson::new(users, &params, total_count)
```

### Response Headers

```
X-Total-Count: 100
X-Total-Pages: 5
X-Current-Page: 1
X-Per-Page: 20
```

## Link Header

Generate RFC 5988 Link headers:

```rust
use paginator_axum::create_link_header;

let link = create_link_header(&params, total_count, "/api/users");
```
