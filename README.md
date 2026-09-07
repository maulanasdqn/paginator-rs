# paginator-rs

Modular Rust pagination library with database and web framework integrations.

## Features

- Page-based, offset/limit, cursor (keyset), and relative cursor pagination
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
paginator-rs = "0.4.0"
```

Add the integration crate you need, for example:

```toml
paginator-sqlx = { version = "0.4.0", features = ["postgres", "runtime-tokio"] }
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

Cursor (keyset) pagination orders rows by the cursor field and selects the rows on the far side of the cursor, so pages stay stable while rows are inserted or deleted. Every database integration supports it.

```rust
use paginator_rs::{CursorValue, PaginatorBuilder};

// Rows after id 42, newest first: WHERE id < 42 ORDER BY id DESC LIMIT 21
let params = PaginatorBuilder::new()
    .per_page(20)
    .sort_by("id")
    .sort_desc()
    .cursor_after("id", CursorValue::Int(42))
    .disable_total_count() // skip COUNT(*)
    .build();
```

The response carries `next_cursor` and `prev_cursor`, derived from the last and first row of the page. Feed one back with `.cursor_from_encoded(cursor)`, or the `cursor` query parameter in the web integrations, to move on. `cursor_before` fetches the page that ends just before a row, so both directions work. Cursors are URL-safe Base64 and validated on decode.

To hand out the first cursor from an ordinary offset page, call `.with_cursors("id")` on the response:

```rust
let page = paginate_query::<_, User>(pool, "SELECT * FROM users", &params)
    .await?
    .with_cursors("id");
// page.meta.next_cursor is set whenever there is a next page
```

Rules and edge cases:

- `sort_by`, when set, must equal the cursor field. `sort_direction` applies as usual and must be sent along with the cursor.
- The cursor field should be unique, such as a primary key. Keyset pagination on a non-unique column skips rows that share the boundary value.
- Cursors are only emitted when the cursor field is present in the serialized row type, so select it.
- `total` and `total_pages` still describe the whole result set, not the rows past the cursor.

### Relative cursor pagination

`page` combines with a cursor as an offset in pages relative to it, so a client can jump several pages ahead of (or behind) a known cursor without walking through them:

```rust
// The third page after id 42: WHERE id > 42 ORDER BY id LIMIT 21 OFFSET 40
let params = PaginatorBuilder::new()
    .per_page(20)
    .page(3)
    .cursor_after("id", CursorValue::Int(42))
    .build();
```

The response's `page` echoes the relative page number, and its cursors point at that page's boundary rows.

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

Cursor pagination adds `next_cursor`/`prev_cursor`, each present only when that side has more rows; with `disable_total_count()`, `total` and `total_pages` are omitted. Web framework integrations also set `X-Total-Count`, `X-Total-Pages`, `X-Current-Page`, and `X-Per-Page` headers.

## Query parameters

```
GET /api/users?page=1&per_page=10&filter=status:eq:active&filter=age:gt:18&search=developer&search_fields=title,bio&sort_by=created_at&sort_direction=desc
```

- `page` — 1-indexed, default 1
- `per_page` — default 20, max 100
- `sort_by` / `sort_direction` — field and `asc`/`desc`
- `filter` — `field:operator:value`, repeatable (AND logic)
- `search` / `search_fields` — query text and comma-separated fields
- `cursor` — a `next_cursor`/`prev_cursor` from a previous response; invalid cursors are rejected with 400

## License

MIT © 2025 Maulana Sodiqin
