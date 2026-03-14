---
title: Security
description: SQL injection prevention and secure cursor encoding
---

## SQL Injection Prevention

All database integrations use **parameterized queries** with bound parameters:

```rust
// Safe: malicious input is escaped as a parameter value
let params = PaginatorBuilder::new()
    .filter_eq("status", FilterValue::String(
        "'; DROP TABLE users; --".to_string()
    ))
    .build();

// SQL: WHERE status = $1
// Parameter: "'; DROP TABLE users; --"
```

### Implementation by Integration

| Integration | Technique |
|-------------|-----------|
| `paginator-sqlx` | SQLx `QueryBuilder` with `.push_bind()` |
| `paginator-sea-orm` | SeaORM's type-safe query builder |
| `paginator-surrealdb` | SurrealDB's parameterized query API |

Filter values, search terms, and sort fields are **never** concatenated into SQL strings.

## Field Name Validation

Use `validate_field_name()` to ensure field names are safe SQL identifiers:

```rust
use paginator_sqlx::validate_field_name;

assert!(validate_field_name("user_name"));    // true
assert!(!validate_field_name("user; DROP"));  // false
```

## Secure Cursor Encoding

Cursors are Base64-encoded JSON objects to prevent tampering:

```rust
// Cursor structure
// { "field": "id", "value": 42, "direction": "after" }

// Encoded: "eyJmaWVsZCI6ImlkIiwidmFsdWUiOjQyLCJkaXJlY3Rpb24iOiJhZnRlciJ9"

// Type-safe decoding with validation
let cursor = Cursor::decode(encoded_cursor)?;
// Returns error if cursor is tampered or invalid
```

## Best Practices

- Always validate user input before building pagination parameters
- Use type-safe filter values (`FilterValue::String`, `FilterValue::Int`, etc.)
- Cursors are automatically validated during decoding
- All database queries use parameterized statements
- No raw SQL concatenation in any integration
- Use `validate_field_name()` for user-provided sort/filter field names
