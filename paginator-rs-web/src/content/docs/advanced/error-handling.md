---
title: Error Handling
description: Comprehensive error types with helpful messages
---

## PaginatorError

The `PaginatorError` enum provides clear, actionable error messages:

```rust
use paginator_rs::{PaginatorError, PaginatorResult};

match result {
    Ok(response) => println!("Success!"),
    Err(PaginatorError::InvalidPage(page)) => {
        eprintln!("Invalid page: {}. Page must be >= 1", page);
    }
    Err(PaginatorError::InvalidPerPage(per_page)) => {
        eprintln!("Invalid per_page: {}. Must be between 1 and 100", per_page);
    }
    Err(PaginatorError::SerializationError(msg)) => {
        eprintln!("Serialization error: {}", msg);
    }
    Err(PaginatorError::Custom(msg)) => {
        eprintln!("Custom error: {}", msg);
    }
}
```

## Error Variants

| Variant | Description |
|---------|-------------|
| `InvalidPage(u32)` | Page number is less than 1 |
| `InvalidPerPage(u32)` | Per-page value is outside 1-100 range |
| `SerializationError(String)` | Failed to serialize/deserialize data |
| `Custom(String)` | Custom error with a message |

## PaginatorResult

A type alias for convenience:

```rust
pub type PaginatorResult<T> = Result<T, PaginatorError>;
```

## Cursor Decode Errors

Cursor decoding can fail with a `String` error:

```rust
let result = CursorBuilder::new()
    .from_encoded("invalid_base64");

match result {
    Ok(builder) => { /* use builder */ }
    Err(e) => eprintln!("Invalid cursor: {}", e),
}
```
