---
title: Search
description: Full-text search across multiple fields
---

paginator-rs supports multi-field full-text search with options for case sensitivity and exact matching.

## Using the Fluent Builder

```rust
use paginator_rs::Paginator;

let params = Paginator::new()
    .page(1)
    .per_page(20)
    .search()
        .query("john")
        .fields(["name", "email", "bio"])
        .apply()
    .build();
```

### Search Options

```rust
// Exact match search
Paginator::new()
    .search()
        .query("john@example.com")
        .fields(["email"])
        .exact(true)
        .apply()
    .build();

// Case-sensitive search
Paginator::new()
    .search()
        .query("John")
        .fields(["name"])
        .case_sensitive(true)
        .apply()
    .build();
```

## Using the Legacy Builder

```rust
use paginator_rs::PaginatorBuilder;

// Fuzzy search (default, case-insensitive)
let params = PaginatorBuilder::new()
    .search("john", vec!["name".to_string(), "email".to_string()])
    .build();

// Exact match
let params = PaginatorBuilder::new()
    .search_exact("john@example.com", vec!["email".to_string()])
    .build();

// Case-sensitive
let params = PaginatorBuilder::new()
    .search_case_sensitive("John", vec!["name".to_string()])
    .build();
```

## Standalone SearchBuilder

```rust
use paginator_rs::SearchBuilder;

let search = SearchBuilder::new()
    .query("developer")
    .fields(["title", "bio", "skills"])
    .build();
```

## SQL Generation

Search is translated to SQL using `ILIKE` (case-insensitive) or `LIKE` (case-sensitive) across the specified fields:

```
WHERE (name ILIKE '%john%' OR email ILIKE '%john%' OR bio ILIKE '%john%')
```

For exact matching:
```
WHERE (name = 'john' OR email = 'john' OR bio = 'john')
```

## Query Parameter Format

When using web framework integrations:

```
GET /api/users?search=john&search_fields=name,email,bio
```
