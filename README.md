# paginator-rs

A comprehensive, modular Rust pagination library with support for multiple databases and web frameworks. Built for production use with a focus on ergonomics, performance, and maintainability.

## ✨ Features

### Core Features
- 🎯 **Flexible Pagination**: Page-based and offset/limit pagination
- 🔧 **Builder Pattern**: Fluent API for constructing pagination parameters
- 📊 **Rich Metadata**: Automatic calculation of total pages, has_next, has_prev
- 🎨 **Sorting Support**: Multi-field sorting with ascending/descending order
- ⚠️ **Error Handling**: Comprehensive error types with helpful messages
- 🔄 **JSON Serialization**: Built-in serde support

### Advanced Features
- 🔑 **Cursor Pagination**: Keyset-based pagination for large datasets with consistent results
- ⚡ **Optional COUNT()**: Skip expensive COUNT queries with `.disable_total_count()`
- 🔒 **SQL Injection Prevention**: Parameterized queries in all database integrations
- 🏗️ **CTE Support**: Common Table Expressions (WITH clauses) work seamlessly
- 🔍 **Advanced Filtering**: 14 filter operators (eq, ne, gt, lt, like, in, between, etc.)
- 🔎 **Full-text Search**: Multi-field fuzzy search with case-sensitive options

### Database Integrations
- **SQLx** (`paginator-sqlx`): PostgreSQL, MySQL, SQLite support
- **SeaORM** (`paginator-sea-orm`): Type-safe ORM pagination with entity support
- **SurrealDB** (`paginator-surrealdb`): Multi-model database with SQL-like queries

### Web Framework Integrations
- **Axum** (`paginator-axum`): Query extractors and JSON responses with headers
- **Rocket** (`paginator-rocket`): Request guards and responders
- **Actix-web** (`paginator-actix`): Extractors, responders, and middleware

## 🧱 Workspace Structure

```
paginator-rs/
├── paginator-rs/         # Core trait and types
├── paginator-utils/      # Shared types (params, response, metadata)
├── paginator-sqlx/       # SQLx database integration
├── paginator-sea-orm/    # SeaORM integration
├── paginator-surrealdb/  # SurrealDB integration
├── paginator-axum/       # Axum web framework integration
├── paginator-rocket/     # Rocket web framework integration
├── paginator-actix/      # Actix-web integration
└── paginator-examples/   # Usage examples
```

## 📦 Installation

### Core Library
```toml
[dependencies]
paginator-rs = "0.2.2"
paginator-utils = "0.2.2"
serde = { version = "1", features = ["derive"] }
```

### With SQLx (PostgreSQL)
```toml
[dependencies]
paginator-sqlx = { version = "0.2.2", features = ["postgres", "runtime-tokio"] }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }
```

### With SeaORM
```toml
[dependencies]
paginator-sea-orm = { version = "0.2.2", features = ["sqlx-postgres", "runtime-tokio"] }
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio"] }
```

### With SurrealDB
```toml
[dependencies]
paginator-surrealdb = { version = "0.2.2", features = ["protocol-ws", "kv-mem"] }
surrealdb = { version = "2.1", features = ["protocol-ws", "kv-mem"] }
```

### With Axum
```toml
[dependencies]
paginator-axum = "0.2.2"
axum = "0.7"
```

### With Rocket
```toml
[dependencies]
paginator-rocket = "0.2.2"
rocket = { version = "0.5", features = ["json"] }
```

### With Actix-web
```toml
[dependencies]
paginator-actix = "0.2.2"
actix-web = "4"
```

## 🚀 Usage Examples

### Basic Pagination

```rust
use paginator_rs::{PaginationParams, PaginatorBuilder, PaginatorTrait};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Using builder pattern
let params = PaginatorBuilder::new()
    .page(1)
    .per_page(20)
    .sort_by("name")
    .sort_asc()
    .build();

// Or create directly
let params = PaginationParams::new(1, 20);
```

### Filtering & Search

```rust
use paginator_rs::{FilterValue, PaginatorBuilder};

// Example 1: Simple filtering
let params = PaginatorBuilder::new()
    .page(1)
    .per_page(20)
    .filter_eq("status", FilterValue::String("active".to_string()))
    .filter_gt("age", FilterValue::Int(18))
    .build();

// Example 2: Advanced filtering with multiple operators
let params = PaginatorBuilder::new()
    .filter_in("role", vec![
        FilterValue::String("admin".to_string()),
        FilterValue::String("moderator".to_string()),
    ])
    .filter_between("created_at",
        FilterValue::String("2024-01-01".to_string()),
        FilterValue::String("2024-12-31".to_string())
    )
    .build();

// Example 3: Full-text search
let params = PaginatorBuilder::new()
    .search("john", vec!["name".to_string(), "email".to_string()])
    .build();

// Example 4: Combined filters and search
let params = PaginatorBuilder::new()
    .page(1)
    .per_page(10)
    .filter_eq("status", FilterValue::String("active".to_string()))
    .filter_gt("age", FilterValue::Int(18))
    .search("developer", vec!["title".to_string(), "bio".to_string()])
    .sort_by("created_at")
    .sort_desc()
    .build();

// Get generated SQL WHERE clause
if let Some(where_clause) = params.to_sql_where() {
    println!("WHERE {}", where_clause);
    // Output: WHERE status = 'active' AND age > 18 AND (title ILIKE '%developer%' OR bio ILIKE '%developer%')
}
```

### Cursor-Based Pagination

Cursor pagination (keyset pagination) provides better performance and consistency for large datasets compared to offset-based pagination.

```rust
use paginator_rs::{PaginatorBuilder, CursorValue, CursorDirection};

// Example 1: First page with cursor support
let params = PaginatorBuilder::new()
    .per_page(20)
    .sort_by("id")
    .sort_asc()
    .build();

// Example 2: Next page using cursor (better than offset!)
let params = PaginatorBuilder::new()
    .per_page(20)
    .sort_by("id")
    .cursor_after("id", CursorValue::Int(42))
    .build();

// Example 3: Previous page
let params = PaginatorBuilder::new()
    .per_page(20)
    .sort_by("id")
    .cursor_before("id", CursorValue::Int(42))
    .build();

// Example 4: Decode from encoded cursor (from API response)
let params = PaginatorBuilder::new()
    .per_page(20)
    .cursor_from_encoded("eyJmaWVsZCI6ImlkIiwidmFsdWUiOjQyLCJkaXJlY3Rpb24iOiJhZnRlciJ9")
    .unwrap()
    .build();

// Example 5: Skip COUNT query for better performance
let params = PaginatorBuilder::new()
    .per_page(20)
    .sort_by("created_at")
    .cursor_after("created_at", CursorValue::String("2024-01-01T00:00:00Z".to_string()))
    .disable_total_count()  // Skip expensive COUNT(*)
    .build();
```

**Cursor Pagination Benefits:**
- ✅ Better performance on large datasets (no OFFSET overhead)
- ✅ Consistent results even with concurrent data modifications
- ✅ No skipped or duplicate rows
- ✅ Works with filters and search
- ✅ Secure Base64-encoded cursor strings

**Available Filter Operators:**
- `filter_eq(field, value)` - Equal (=)
- `filter_ne(field, value)` - Not equal (!=)
- `filter_gt(field, value)` - Greater than (>)
- `filter_lt(field, value)` - Less than (<)
- `filter_gte(field, value)` - Greater than or equal (>=)
- `filter_lte(field, value)` - Less than or equal (<=)
- `filter_like(field, pattern)` - SQL LIKE pattern matching
- `filter_ilike(field, pattern)` - Case-insensitive LIKE
- `filter_in(field, values)` - IN array
- `filter_between(field, min, max)` - BETWEEN min AND max
- `filter_is_null(field)` - IS NULL
- `filter_is_not_null(field)` - IS NOT NULL

**Search Options:**
- `search(query, fields)` - Case-insensitive fuzzy search
- `search_exact(query, fields)` - Exact match search
- `search_case_sensitive(query, fields)` - Case-sensitive search

### With Axum

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
    let users = vec![/* fetch from database */];

    // Automatically adds pagination headers
    PaginatedJson::new(users, &params, 100)
}

let app = Router::new().route("/users", get(get_users));
```

### With SQLx (PostgreSQL)

```rust
use paginator_sqlx::postgres::paginate_query;
use paginator_rs::PaginatorBuilder;
use sqlx::PgPool;

#[derive(sqlx::FromRow, serde::Serialize)]
struct User {
    id: i32,
    name: String,
}

async fn get_users(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .sort_by("created_at")
        .sort_desc()
        .build();

    let result = paginate_query::<_, User>(
        pool,
        "SELECT id, name FROM users WHERE active = true",
        &params,
    ).await?;

    println!("Page {}/{}", result.meta.page, result.meta.total_pages);
    println!("Total users: {}", result.meta.total);

    Ok(())
}
```

### With SeaORM

```rust
use paginator_sea_orm::PaginateSeaOrm;
use paginator_rs::PaginationParams;
use sea_orm::{EntityTrait, Database};

async fn get_users(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let params = PaginationParams::new(1, 20);

    let result = User::find()
        .filter(user::Column::Active.eq(true))
        .paginate_with(db, &params)
        .await?;

    println!("Found {} users", result.data.len());
    Ok(())
}
```

### With SurrealDB

```rust
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;
use paginator_surrealdb::{paginate_query, paginate_table, QueryBuilder};
use paginator_rs::PaginatorBuilder;

#[derive(serde::Deserialize, serde::Serialize)]
struct User {
    id: String,
    name: String,
    email: String,
    active: bool,
}

async fn get_users() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to SurrealDB
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.use_ns("test").use_db("test").await?;

    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .sort_by("name")
        .sort_asc()
        .build();

    // Option 1: Using raw query
    let result = paginate_query::<User, _>(
        &db,
        "SELECT * FROM users WHERE active = true",
        &params,
    ).await?;

    // Option 2: Using table helper
    let result = paginate_table::<User, _>(
        &db,
        "users",
        Some("active = true"),
        &params,
    ).await?;

    // Option 3: Using query builder
    let result = QueryBuilder::new()
        .select("*")
        .from("users")
        .where_clause("active = true")
        .and("age > 18")
        .paginate::<User, _>(&db, &params)
        .await?;

    println!("Page {}/{}", result.meta.page, result.meta.total_pages);
    println!("Total users: {}", result.meta.total);

    Ok(())
}
```

### With Rocket

```rust
use rocket::{get, routes};
use paginator_rocket::{Pagination, PaginatedJson};

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
}

#[get("/users")]
async fn get_users(pagination: Pagination) -> PaginatedJson<User> {
    let users = vec![/* ... */];
    PaginatedJson::new(users, &pagination.params, 100)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/api", routes![get_users])
}
```

### With Actix-web

```rust
use actix_web::{get, web, App};
use paginator_actix::{PaginationQuery, PaginatedJson};

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
    let users = vec![/* ... */];

    PaginatedJson::new(users, &params, 100)
}
```

## 🧪 Response Format

### Standard Pagination Response

```json
{
  "data": [
    { "id": 1, "name": "Alice", "email": "alice@example.com" },
    { "id": 2, "name": "Bob", "email": "bob@example.com" }
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

### Cursor Pagination Response

When using cursor pagination with `.disable_total_count()`:

```json
{
  "data": [
    { "id": 43, "name": "Charlie", "email": "charlie@example.com" },
    { "id": 44, "name": "Diana", "email": "diana@example.com" }
  ],
  "meta": {
    "page": 1,
    "per_page": 20,
    "has_next": true,
    "has_prev": false,
    "next_cursor": "eyJmaWVsZCI6ImlkIiwidmFsdWUiOjQ0LCJkaXJlY3Rpb24iOiJhZnRlciJ9",
    "prev_cursor": "eyJmaWVsZCI6ImlkIiwidmFsdWUiOjQzLCJkaXJlY3Rpb24iOiJiZWZvcmUifQ=="
  }
}
```

**Note**: When `disable_total_count()` is used, `total` and `total_pages` fields are omitted from the response for better performance.

### HTTP Headers (Web Framework Integrations)

```
X-Total-Count: 100
X-Total-Pages: 5
X-Current-Page: 1
X-Per-Page: 20
```

**Note**: `X-Total-Count` and `X-Total-Pages` headers are only included when `total` is available (not using `disable_total_count()`).

## 🎯 Query Parameters

### Basic Pagination & Sorting
```
GET /api/users?page=2&per_page=20&sort_by=name&sort_direction=asc
```

- `page`: Page number (1-indexed, default: 1)
- `per_page`: Items per page (default: 20, max: 100)
- `sort_by`: Field to sort by (optional)
- `sort_direction`: `asc` or `desc` (optional)

### With Filters
```
GET /api/users?page=1&filter=status:eq:active&filter=age:gt:18&filter=role:in:admin,moderator
```

- `filter`: Filter in format `field:operator:value`
- Multiple filters can be combined (AND logic)

**Filter Format Examples:**
- `status:eq:active` - Equal
- `age:gt:18` - Greater than
- `age:between:18,65` - Between
- `role:in:admin,moderator,user` - In array
- `name:like:%john%` - LIKE pattern
- `deleted_at:is_null` - IS NULL

### With Search
```
GET /api/users?search=john&search_fields=name,email,bio
```

- `search`: Search query text
- `search_fields`: Comma-separated list of fields to search in

### Combined Example
```
GET /api/users?page=1&per_page=10&filter=status:eq:active&filter=age:gt:18&search=developer&search_fields=title,bio&sort_by=created_at&sort_direction=desc
```

## 🔧 Builder Pattern

```rust
use paginator_rs::PaginatorBuilder;

let params = PaginatorBuilder::new()
    .page(2)
    .per_page(50)
    .sort_by("created_at")
    .sort_desc()
    .build();
```

## ⚠️ Error Handling

```rust
use paginator_rs::{PaginatorError, PaginatorResult};

// Errors are comprehensive and helpful
match result {
    Ok(response) => println!("Success!"),
    Err(PaginatorError::InvalidPage(page)) => {
        eprintln!("Invalid page: {}. Page must be >= 1", page);
    }
    Err(PaginatorError::InvalidPerPage(per_page)) => {
        eprintln!("Invalid per_page: {}. Must be between 1 and 100", per_page);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## 🏗️ Architecture

- **Easy to Use**: Builder pattern and sensible defaults
- **Easy to Debug**: Comprehensive error messages and type safety
- **Easy to Maintain**: Modular crate structure with clear separation of concerns

## 🔒 Security

### SQL Injection Prevention

All database integrations use **parameterized queries** with bound parameters to prevent SQL injection attacks:

```rust
// ✅ SAFE: All filter values are bound parameters
let params = PaginatorBuilder::new()
    .filter_eq("status", FilterValue::String("'; DROP TABLE users; --".to_string()))
    .build();

// The malicious input is safely escaped as a parameter value
// SQL: WHERE status = $1  (with parameter: "'; DROP TABLE users; --")
```

**Implementation Details:**
- `paginator-sqlx`: Uses SQLx's `QueryBuilder` with `.push_bind()` for all values
- `paginator-sea-orm`: Uses SeaORM's type-safe query builder
- `paginator-surrealdb`: Uses SurrealDB's parameterized query API
- Filter values, search terms, and sort fields are never concatenated into SQL strings

### Secure Cursor Encoding

Cursors are Base64-encoded JSON objects to prevent tampering:

```rust
// Cursor structure: { "field": "id", "value": 42, "direction": "after" }
// Encoded: "eyJmaWVsZCI6ImlkIiwidmFsdWUiOjQyLCJkaXJlY3Rpb24iOiJhZnRlciJ9"

// ✅ Type-safe decoding with validation
let cursor = Cursor::decode(encoded_cursor)?;
// Returns error if cursor is tampered or invalid
```

### Best Practices

- ✅ Always validate user input before building pagination parameters
- ✅ Use type-safe filter values (`FilterValue::String`, `FilterValue::Int`, etc.)
- ✅ Cursors are automatically validated during decoding
- ✅ All database queries use parameterized statements
- ✅ No raw SQL concatenation in any integration

## 📝 Examples

Run the examples:

```bash
cargo run --package paginator-examples --bin example
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT © 2025 Maulana Sodiqin

## 🔗 Links

- [Repository](https://github.com/maulanasdqn/paginator-rs)
- [Documentation](https://docs.rs/paginator-rs) (coming soon)
- [crates.io](https://crates.io/crates/paginator-rs) (coming soon)
