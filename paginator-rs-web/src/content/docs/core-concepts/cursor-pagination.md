---
title: Cursor Pagination
description: Keyset-based pagination for large datasets
---

Cursor pagination (keyset pagination) provides better performance and consistency for large datasets compared to offset-based pagination.

## Why Cursor Pagination?

| Feature | Offset-based | Cursor-based |
|---------|-------------|-------------|
| Performance on large datasets | Degrades with higher pages | Consistent |
| Concurrent modifications | May skip/duplicate rows | Consistent results |
| Random page access | Supported | Not supported |
| Implementation complexity | Simple | Moderate |

## Using the Fluent Builder

```rust
use paginator_rs::{Paginator, CursorValue};

// First page (no cursor needed)
let params = Paginator::new()
    .per_page(20)
    .sort().asc("id")
    .build();

// Next page using cursor
let params = Paginator::new()
    .per_page(20)
    .cursor()
        .after("id", CursorValue::Int(42))
        .apply()
    .build();

// Previous page
let params = Paginator::new()
    .per_page(20)
    .cursor()
        .before("id", CursorValue::Int(42))
        .apply()
    .build();
```

## Using the Legacy Builder

```rust
use paginator_rs::{PaginatorBuilder, CursorValue};

// Next page
let params = PaginatorBuilder::new()
    .per_page(20)
    .sort_by("id")
    .cursor_after("id", CursorValue::Int(42))
    .build();

// Previous page
let params = PaginatorBuilder::new()
    .per_page(20)
    .sort_by("id")
    .cursor_before("id", CursorValue::Int(42))
    .build();
```

## Cursor Values

The `CursorValue` enum supports multiple types:

```rust
use paginator_rs::CursorValue;

CursorValue::String("2024-01-01T00:00:00Z".to_string())
CursorValue::Int(42)
CursorValue::Float(3.14)
CursorValue::Uuid("550e8400-e29b-41d4-a716-446655440000".to_string())
```

## Encoded Cursors

Cursors returned in API responses are Base64-encoded JSON. You can decode them:

```rust
// From API response meta.next_cursor
let params = PaginatorBuilder::new()
    .per_page(20)
    .cursor_from_encoded("eyJmaWVsZCI6ImlkIiwidmFsdWUiOjQyLCJkaXJlY3Rpb24iOiJhZnRlciJ9")
    .unwrap()
    .build();

// Or with fluent builder
let params = Paginator::new()
    .per_page(20)
    .cursor()
        .from_encoded("eyJmaWVsZCI6ImlkIiwidmFsdWUiOjQyLCJkaXJlY3Rpb24iOiJhZnRlciJ9")
        .unwrap()
        .apply()
    .build();
```

## Disabling Total Count

For optimal performance with cursor pagination, skip the expensive `COUNT(*)` query:

```rust
let params = Paginator::new()
    .per_page(20)
    .sort().asc("id")
    .disable_total_count()
    .build();
```

When total count is disabled, `meta.total` and `meta.total_pages` will be `None`, but `meta.has_next` and `meta.has_prev` will still work correctly.

## Response with Cursors

```json
{
  "data": [...],
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
