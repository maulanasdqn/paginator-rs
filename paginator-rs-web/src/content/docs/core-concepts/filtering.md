---
title: Filtering
description: Apply advanced filters with 14 operators
---

paginator-rs supports 14 filter operators for building complex query conditions.

## Filter Operators

| Method | SQL | Description |
|--------|-----|-------------|
| `eq(field, value)` | `= value` | Equal |
| `ne(field, value)` | `!= value` | Not equal |
| `gt(field, value)` | `> value` | Greater than |
| `lt(field, value)` | `< value` | Less than |
| `gte(field, value)` | `>= value` | Greater than or equal |
| `lte(field, value)` | `<= value` | Less than or equal |
| `like(field, pattern)` | `LIKE pattern` | Pattern matching |
| `ilike(field, pattern)` | `ILIKE pattern` | Case-insensitive pattern matching |
| `r#in(field, values)` | `IN (values)` | In array |
| `not_in(field, values)` | `NOT IN (values)` | Not in array |
| `between(field, min, max)` | `BETWEEN min AND max` | Between range |
| `is_null(field)` | `IS NULL` | Is null |
| `is_not_null(field)` | `IS NOT NULL` | Is not null |
| `contains(field, value)` | `LIKE %value%` | Contains substring |

## Using the Fluent Builder

```rust
use paginator_rs::Paginator;

let params = Paginator::new()
    .page(1)
    .per_page(20)
    .filter()
        .eq("status", "active")
        .gt("age", 18)
        .like("name", "%john%")
        .apply()
    .build();
```

## Using the Legacy Builder

```rust
use paginator_rs::{PaginatorBuilder, FilterValue};

let params = PaginatorBuilder::new()
    .filter_eq("status", FilterValue::String("active".to_string()))
    .filter_gt("age", FilterValue::Int(18))
    .filter_like("name", "%john%".to_string())
    .filter_in("role", vec![
        FilterValue::String("admin".to_string()),
        FilterValue::String("moderator".to_string()),
    ])
    .filter_between("created_at",
        FilterValue::String("2024-01-01".to_string()),
        FilterValue::String("2024-12-31".to_string()),
    )
    .filter_is_null("deleted_at")
    .build();
```

## Standalone FilterBuilder

```rust
use paginator_rs::FilterBuilder;

let filters = FilterBuilder::new()
    .eq("status", "active")
    .gt("age", 18)
    .build();
```

## Filter Values

The `FilterValue` enum supports multiple types:

```rust
use paginator_rs::FilterValue;

FilterValue::String("active".to_string())
FilterValue::Int(42)
FilterValue::Float(3.14)
FilterValue::Bool(true)
FilterValue::Array(vec![FilterValue::Int(1), FilterValue::Int(2)])
FilterValue::Null
```

## SQL Generation

Filters are automatically converted to SQL WHERE clauses:

```rust
let params = PaginatorBuilder::new()
    .filter_eq("status", FilterValue::String("active".to_string()))
    .filter_gt("age", FilterValue::Int(18))
    .build();

if let Some(where_clause) = params.to_sql_where() {
    println!("WHERE {}", where_clause);
    // Output: WHERE status = 'active' AND age > 18
}
```

Multiple filters are combined with AND logic.
