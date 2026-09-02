//! [zod-rs](https://crates.io/crates/zod-rs) integration for
//! [paginator-rs](https://crates.io/crates/paginator-rs).
//!
//! Two directions:
//!
//! - **Server-side input validation** ([`PaginationSchema`]): validate raw
//!   pagination query JSON against a zod-rs schema and get a normalized
//!   [`PaginationParams`](paginator_utils::PaginationParams) back, with rich,
//!   path-aware, localizable validation errors.
//! - **Client-side TypeScript** ([`typescript`]): emit a Zod schema for the
//!   paginated response envelope so frontends get typed, validated responses.
//!
//! ```
//! use paginator_zod::PaginationSchema;
//! use serde_json::json;
//!
//! let schema = PaginationSchema::new()
//!     .max_per_page(100)
//!     .allowed_sort_fields(["name", "created_at"]);
//!
//! let params = schema
//!     .validate(&json!({ "page": 1, "per_page": 20, "sort_by": "name", "sort_direction": "asc" }))
//!     .unwrap();
//! assert_eq!(params.per_page, 20);
//!
//! // Rejected: per_page over the max, and an unlisted sort field.
//! assert!(schema.validate(&json!({ "per_page": 500 })).is_err());
//! assert!(schema.validate(&json!({ "sort_by": "password" })).is_err());
//! ```

mod validate;

pub mod typescript;

pub use validate::PaginationSchema;

// Re-exported so callers can render/localize errors without a direct zod-rs-util dep.
pub use zod_rs_util::{Locale, ValidateResult, ValidationError, ValidationResult};
