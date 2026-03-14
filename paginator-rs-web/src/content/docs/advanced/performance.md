---
title: Performance
description: Tips for optimizing pagination performance
---

## Disable Total Count

The most impactful optimization is skipping the `COUNT(*)` query, which can be expensive on large tables:

```rust
use paginator_rs::Paginator;

let params = Paginator::new()
    .per_page(20)
    .disable_total_count()
    .build();
```

When disabled:
- `meta.total` and `meta.total_pages` will be `None`
- `meta.has_next` and `meta.has_prev` still work correctly
- The database only executes one query instead of two

## Use Cursor Pagination

For large datasets, cursor pagination outperforms offset-based pagination:

```rust
use paginator_rs::{Paginator, CursorValue};

// First page
let params = Paginator::new()
    .per_page(20)
    .sort().asc("id")
    .disable_total_count()
    .build();

// Subsequent pages use cursor
let params = Paginator::new()
    .per_page(20)
    .cursor()
        .after("id", CursorValue::Int(last_id))
        .apply()
    .disable_total_count()
    .build();
```

### Why Cursor Pagination is Faster

| Offset-based | Cursor-based |
|--------------|-------------|
| `OFFSET 10000 LIMIT 20` scans 10,020 rows | `WHERE id > 42 LIMIT 20` uses index |
| Gets slower on later pages | Consistent performance |
| May return inconsistent results with concurrent writes | Always consistent |

## Index Your Sort/Filter Fields

Ensure database indexes exist on fields used for:
- Sorting (`sort_by`)
- Filtering (filter fields)
- Cursor pagination (cursor field)

```sql
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_name_email ON users(name, email); -- For search
```

## CTE Support

CTE queries (WITH clauses) work seamlessly and can improve performance for complex queries:

```rust
let result = paginate_query::<_, User>(
    pool,
    "WITH active AS (
        SELECT * FROM users WHERE active = true
    )
    SELECT * FROM active",
    &params,
).await?;
```
