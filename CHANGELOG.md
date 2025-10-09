# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-10-09

### Added

#### Core Features
- **Comprehensive Filtering System** - Added support for 14 filter operators:
  - Comparison: `Eq`, `Ne`, `Gt`, `Lt`, `Gte`, `Lte`
  - Pattern Matching: `Like`, `ILike`, `Contains`
  - List Operations: `In`, `NotIn`
  - Null Checks: `IsNull`, `IsNotNull`
  - Range: `Between`
- **Search Functionality** - Full-text search with:
  - Multi-field search support
  - Case-sensitive and case-insensitive modes
  - Exact match and fuzzy matching options
  - Configurable search fields
- **Builder Pattern Enhancements** - Added fluent API methods:
  - `filter()`, `filter_eq()`, `filter_ne()`, `filter_gt()`, `filter_lt()`
  - `filter_gte()`, `filter_lte()`, `filter_like()`, `filter_in()`
  - `filter_between()`, `filter_contains()`
  - `search()` with field specification

#### Web Framework Integrations (NEW)
- **Axum Integration** (`paginator-axum` v0.1.1)
  - `PaginationQuery` extractor for query parameters
  - `PaginatedJson` responder with automatic headers
  - Advanced filter and search parameter parsing
  - Link header generation (RFC 8288 compliant)

- **Rocket Integration** (`paginator-rocket` v0.1.1)
  - `Pagination` request guard
  - `PaginatedJson` responder
  - Automatic query parameter extraction
  - Response headers for pagination metadata

- **Actix-web Integration** (`paginator-actix` v0.1.1)
  - `PaginationQuery` extractor
  - `PaginatedJson` responder
  - Middleware for automatic pagination
  - Header-based metadata delivery

#### Database Integrations (NEW)
- **SQLx Integration** (`paginator-sqlx` v0.1.1)
  - Support for PostgreSQL, MySQL, and SQLite
  - Automatic SQL generation with filters and search
  - Feature-gated database backends
  - Runtime selection (tokio/async-std)

- **SeaORM Integration** (`paginator-sea-orm` v0.1.1)
  - Type-safe entity pagination
  - Query builder integration
  - Filter and search translation to SeaORM queries
  - Multi-database support via SeaORM

- **SurrealDB Integration** (`paginator-surrealdb` v0.1.1)
  - Native SurrealDB query pagination
  - Multi-model database support
  - Protocol-agnostic (WebSocket/HTTP)
  - Feature-gated backends (mem/rocksdb)

#### Testing & Quality
- Added 27 comprehensive edge case tests:
  - 7 pagination edge cases (empty, single item, boundaries, beyond total)
  - 8 filtering edge cases (all operators, multiple filters, AND logic)
  - 6 search edge cases (case sensitivity, multi-field, exact match)
  - 3 sorting edge cases (asc/desc, combined with filters)
  - 3 combined operation tests (filter + search + sort + pagination)
- Total test count increased from 2 to 30 tests
- Added large dataset pagination test (100 items)

### Changed
- **BREAKING**: `PaginationParams` now includes `filters` and `search` fields
  - Migration: Existing code using struct initialization must add `filters: Vec::new()` and `search: None`
  - Builder pattern usage is unaffected
- Updated workspace dependencies to version 0.2.0 for core crates
- Enhanced documentation with comprehensive examples for all integrations
- Improved README with installation instructions for all integrations

### Fixed
- Fixed missing `filters` and `search` field initializations in:
  - `paginator-rocket/src/lib.rs` (line 99-106)
  - `paginator-actix/src/lib.rs` (lines 85-92, 106-113)
- Removed unused imports causing warnings in:
  - `paginator-sqlx`: 4 warnings eliminated
  - `paginator-axum`: 2 warnings eliminated
  - `paginator-rocket`: 3 warnings eliminated
  - `paginator-actix`: 2 warnings eliminated
- Fixed test compilation errors in `paginator-examples`
- Achieved zero compiler warnings across entire workspace

### Documentation
- Updated README with comprehensive usage examples
- Added integration-specific documentation
- Added filter and search feature documentation
- Included real-world examples for all database and web framework integrations

## [0.1.2] - 2025-10-09

### Changed
- Updated documentation
- Version bump for crates.io compatibility

## [0.1.0] - 2025-10-09

### Added
- Initial release
- Basic pagination trait (`PaginatorTrait`)
- Core types: `PaginationParams`, `PaginatorResponse`, `PaginatorResponseMeta`
- Builder pattern for pagination parameters
- Sorting support (ascending/descending)
- JSON serialization support
- Error handling with `PaginatorError`
- Basic examples

[0.2.0]: https://github.com/maulanasdqn/paginator-rs/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/maulanasdqn/paginator-rs/compare/v0.1.0...v0.1.2
[0.1.0]: https://github.com/maulanasdqn/paginator-rs/releases/tag/v0.1.0
