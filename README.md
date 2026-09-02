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
| `paginator-zod` | [zod-rs](https://github.com/maulanasdqn/zod-rs) validation of pagination input and TypeScript response schemas |

## Installation

```toml
[dependencies]
paginator-rs = "0.3.2"
```

Add the integration crate you need, for example:

```toml
paginator-sqlx = { version = "0.3.2", features = ["postgres", "runtime-tokio"] }
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

SeaORM, SurrealDB, Rocket, and Actix-web work the same way; [`paginator-examples`](paginator-examples) has a runnable example for every feature and integration:

```bash
cargo run -p paginator-examples --bin basic           # params, sorting, response meta
cargo run -p paginator-examples --bin builders        # every builder API
cargo run -p paginator-examples --bin filters         # all 14 filter operators
cargo run -p paginator-examples --bin search          # all search modes
cargo run -p paginator-examples --bin cursors         # cursor pagination end to end
cargo run -p paginator-examples --bin errors          # error handling
cargo run -p paginator-examples --bin sqlx_sqlite     # SQLx (in-memory SQLite)
cargo run -p paginator-examples --bin sea_orm_sqlite  # SeaORM 2.0 (in-memory SQLite)
cargo run -p paginator-examples --bin surrealdb_mem   # SurrealDB (in-memory engine)
cargo run -p paginator-examples --bin axum_server     # Axum HTTP server
cargo run -p paginator-examples --bin actix_server    # Actix-web HTTP server
cargo run -p paginator-examples --bin rocket_server   # Rocket HTTP server
cargo run -p paginator-zod --example validate_and_codegen  # zod-rs input validation + TS codegen
```

### Validating input with zod-rs

`paginator-zod` validates raw pagination query JSON against a [zod-rs](https://github.com/maulanasdqn/zod-rs) schema before it becomes `PaginationParams`, with path-aware, localizable errors. It enforces `per_page` bounds and, optionally, allow-lists for sort and filter fields.

```rust
use paginator_zod::PaginationSchema;
use serde_json::json;

let schema = PaginationSchema::new()
    .max_per_page(100)
    .allowed_sort_fields(["name", "created_at"])
    .allowed_filter_fields(["status", "age"]);

// Ok -> PaginationParams, ready to paginate
let params = schema.validate(&json!({
    "page": 1,
    "per_page": 20,
    "sort_by": "created_at",
    "sort_direction": "desc",
    "filters": [{ "field": "status", "operator": "eq", "value": "active" }]
}))?;

// Err -> "per_page: Too big: expected number to have <= 100"
schema.validate(&json!({ "per_page": 500 })).unwrap_err();
```

It also emits a Zod schema for the response envelope so frontends get typed, validated responses:

```rust
println!("{}", paginator_zod::typescript::response_module_ts());
// export const paginated = <T extends z.ZodTypeAny>(item: T) =>
//   z.object({ data: z.array(item), meta: PaginationMetaSchema });
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
