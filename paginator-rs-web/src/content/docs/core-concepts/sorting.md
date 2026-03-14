---
title: Sorting
description: Sort results by any field in ascending or descending order
---

## Using the Fluent Builder

```rust
use paginator_rs::Paginator;

// Sort ascending
let params = Paginator::new()
    .sort().asc("name")
    .build();

// Sort descending
let params = Paginator::new()
    .sort().desc("created_at")
    .build();
```

## Using the Legacy Builder

```rust
use paginator_rs::PaginatorBuilder;

let params = PaginatorBuilder::new()
    .sort_by("created_at")
    .sort_desc()
    .build();
```

## Sort Direction

The `SortDirection` enum has two variants:

```rust
use paginator_rs::SortDirection;

SortDirection::Asc   // Ascending order
SortDirection::Desc  // Descending order
```

## Direct Construction

```rust
use paginator_rs::{PaginationParams, SortDirection};

let params = PaginationParams::new(1, 20)
    .with_sort("created_at")
    .with_direction(SortDirection::Desc);
```

## Query Parameter Format

When using web framework integrations, sorting is controlled via query parameters:

```
GET /api/users?sort_by=name&sort_direction=asc
GET /api/users?sort_by=created_at&sort_direction=desc
```
