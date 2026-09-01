# paginator-rs

Modular Rust pagination library with database and web framework integrations.

## Features

- Page-based, offset/limit, and cursor (keyset) pagination
- Builder API with multi-field sorting
- Filtering with 14 operators (eq, ne, gt, lt, gte, lte, like, ilike, in, between, is_null, is_not_null) and multi-field search
- Optional `COUNT(*)` skipping via `.disable_total_count()`
- Parameterized queries in all database integrations
- Serde serialization built in

## Crates

| Crate | Purpose |
|---|---|
| `paginator-rs` | Core trait and types |
| `paginator-utils` | Shared types (params, response, metadata) |
| `paginator-sqlx` | SQLx (PostgreSQL, MySQL, SQLite) |
| `paginator-sea-orm` | SeaORM |
| `paginator-surrealdb` | SurrealDB |
| `paginator-axum` | Axum extractors and responses |
| `paginator-rocket` | Rocket guards and responders |
| `paginator-actix` | Actix-web extractors and responders |

## Installation

```toml
[dependencies]
paginator-rs = "0.3.0"
```

Add the integration crate you need, for example:

```toml
paginator-sqlx = { version = "0.3.0", features = ["postgres", "runtime-tokio"] }
```

## Usage

### Building parameters

```rust
use paginator_rs::{FilterValue, PaginatorBuilder};

let params = PaginatorBuilder::new()
    .page(1)
    .per_page(20)
    .filter_eq("status", FilterValue::String("active".to_string()))
    .filter_gt("age", FilterValue::Int(18))
    .search("developer", vec!["title".to_string(), "bio".to_string()])
    .sort_by("created_at")
    .sort_desc()
    .build();
```

### Cursor pagination

```rust
use paginator_rs::{CursorValue, PaginatorBuilder};

let params = PaginatorBuilder::new()
    .per_page(20)
    .sort_by("id")
    .cursor_after("id", CursorValue::Int(42))
    .disable_total_count() // skip COUNT(*)
    .build();
```

Cursors are Base64-encoded and validated on decode; use `.cursor_from_encoded(cursor)` to resume from an API response.

### SQLx

```rust
use paginator_rs::PaginatorBuilder;
use paginator_sqlx::postgres::paginate_query;

let params = PaginatorBuilder::new().page(1).per_page(10).build();

let result = paginate_query::<_, User>(
    pool,
    "SELECT id, name FROM users WHERE active = true",
    &params,
).await?;

println!("Page {}/{}", result.meta.page, result.meta.total_pages);
```

### Axum

```rust
use paginator_axum::{PaginatedJson, PaginationQuery};

async fn get_users(
    PaginationQuery(params): PaginationQuery,
) -> PaginatedJson<User> {
    let users = vec![/* fetch from database */];
    PaginatedJson::new(users, &params, 100)
}
```

SeaORM, SurrealDB, Rocket, and Actix-web work the same way; see [`paginator-examples`](paginator-examples) for full examples of each integration:

```bash
cargo run --package paginator-examples --bin example
```

## Response format

```json
{
  "data": [
    { "id": 1, "name": "Alice" }
  ],
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5,
    "has_next": true,
    "has_prev": false
  }
}
```

Cursor pagination adds `next_cursor`/`prev_cursor`; with `disable_total_count()`, `total` and `total_pages` are omitted. Web framework integrations also set `X-Total-Count`, `X-Total-Pages`, `X-Current-Page`, and `X-Per-Page` headers.

## Query parameters

```
GET /api/users?page=1&per_page=10&filter=status:eq:active&filter=age:gt:18&search=developer&search_fields=title,bio&sort_by=created_at&sort_direction=desc
```

- `page` — 1-indexed, default 1
- `per_page` — default 20, max 100
- `sort_by` / `sort_direction` — field and `asc`/`desc`
- `filter` — `field:operator:value`, repeatable (AND logic)
- `search` / `search_fields` — query text and comma-separated fields

## License

MIT © 2025 Maulana Sodiqin
