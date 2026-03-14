---
title: Response Format
description: Understanding the paginated response structure
---

## Standard Response

All paginator-rs integrations return a `PaginatorResponse<T>`:

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

## Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `page` | `u32` | Current page number (1-indexed) |
| `per_page` | `u32` | Items per page |
| `total` | `Option<u32>` | Total number of items (None if disabled) |
| `total_pages` | `Option<u32>` | Total number of pages (None if disabled) |
| `has_next` | `bool` | Whether there are more pages |
| `has_prev` | `bool` | Whether there are previous pages |
| `next_cursor` | `Option<String>` | Base64-encoded cursor for next page |
| `prev_cursor` | `Option<String>` | Base64-encoded cursor for previous page |

## With Disabled Total Count

When using `.disable_total_count()`, `total` and `total_pages` are omitted:

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

## HTTP Headers

Web framework integrations automatically add pagination headers:

```
X-Total-Count: 100
X-Total-Pages: 5
X-Current-Page: 1
X-Per-Page: 20
```

`X-Total-Count` and `X-Total-Pages` headers are only included when `total` is available.

## Rust Types

```rust
use paginator_rs::{PaginatorResponse, PaginatorResponseMeta};

// The response wrapper
pub struct PaginatorResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginatorResponseMeta,
}

// The metadata
pub struct PaginatorResponseMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: Option<u32>,
    pub total_pages: Option<u32>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}
```

### Constructors

```rust
// Standard with total count
let meta = PaginatorResponseMeta::new(1, 20, 100);

// Without total count
let meta = PaginatorResponseMeta::new_without_total(1, 20, true);

// With cursors
let meta = PaginatorResponseMeta::new_with_cursors(
    1, 20, 100, true,
    Some("next_cursor_string".to_string()),
    Some("prev_cursor_string".to_string()),
);
```
