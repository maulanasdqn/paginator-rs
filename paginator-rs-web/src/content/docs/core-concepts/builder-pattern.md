---
title: Builder Pattern
description: Build pagination parameters with a fluent, chainable API
---

paginator-rs provides two builder APIs: the modern **fluent builder** (`Paginator`) and the **legacy builder** (`PaginatorBuilder`).

## Fluent Builder (Recommended)

The `Paginator` struct provides chainable sub-builders for sorting, filtering, search, and cursor pagination.

```rust
use paginator_rs::Paginator;

let params = Paginator::new()
    .page(2)
    .per_page(50)
    .sort().desc("created_at")
    .filter()
        .eq("status", "active")
        .gt("age", 18)
        .apply()
    .search()
        .query("developer")
        .fields(["title", "bio"])
        .apply()
    .build();
```

### Sub-builders

Each sub-builder returns control back to the parent when you call `.apply()`:

#### SortBuilder

```rust
// Sort ascending
Paginator::new().sort().asc("name").build();

// Sort descending
Paginator::new().sort().desc("created_at").build();
```

#### FilterBuilder

```rust
Paginator::new()
    .filter()
        .eq("status", "active")
        .gt("age", 18)
        .like("name", "%john%")
        .between("created_at", "2024-01-01", "2024-12-31")
        .is_null("deleted_at")
        .apply()
    .build();
```

#### SearchBuilder

```rust
Paginator::new()
    .search()
        .query("john")
        .fields(["name", "email", "bio"])
        .exact(false)
        .case_sensitive(false)
        .apply()
    .build();
```

#### CursorBuilder

```rust
use paginator_rs::CursorValue;

Paginator::new()
    .cursor()
        .after("id", CursorValue::Int(42))
        .apply()
    .build();
```

## Standalone Builders

Sub-builders can also be used independently:

```rust
use paginator_rs::{FilterBuilder, SearchBuilder, CursorBuilder};

// Build filters independently
let filters = FilterBuilder::new()
    .eq("status", "active")
    .gt("age", 18)
    .build();

// Build search independently
let search = SearchBuilder::new()
    .query("john")
    .fields(["name", "email"])
    .build();

// Build cursor independently
let cursor = CursorBuilder::new()
    .after("id", CursorValue::Int(42))
    .build();
```

## Legacy Builder

The `PaginatorBuilder` provides a flat API for backward compatibility:

```rust
use paginator_rs::{PaginatorBuilder, FilterValue};

let params = PaginatorBuilder::new()
    .page(2)
    .per_page(50)
    .sort_by("created_at")
    .sort_desc()
    .filter_eq("status", FilterValue::String("active".to_string()))
    .filter_gt("age", FilterValue::Int(18))
    .search("developer", vec!["title".to_string(), "bio".to_string()])
    .build();
```

## Direct Construction

You can also create `PaginationParams` directly:

```rust
use paginator_rs::PaginationParams;

let params = PaginationParams::new(1, 20);
```

### Chaining with methods

```rust
let params = PaginationParams::new(1, 20)
    .with_sort("created_at")
    .with_direction(SortDirection::Desc)
    .with_filter(Filter {
        field: "status".to_string(),
        operator: FilterOperator::Eq,
        value: FilterValue::String("active".to_string()),
    });
```
