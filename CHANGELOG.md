# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-10-24

### Added

#### Cursor-based Pagination
- **Keyset Pagination System** - Implemented cursor-based pagination for consistent results:
  - Added `Cursor`, `CursorDirection`, and `CursorValue` types in `paginator-utils`
  - Base64 encoding/decoding for secure cursor transmission via `.encode()` and `.decode()`
  - Builder methods: `.cursor()`, `.cursor_after()`, `.cursor_before()`, `.cursor_from_encoded()`
  - Response metadata includes `next_cursor` and `prev_cursor` fields
  - Implemented in `paginator-sqlx` (Postgres, MySQL, SQLite)
  - Implemented in `paginator-sea-orm` with SeaORM query builder integration
  - Implemented in `paginator-surrealdb` with SurrealQL support
  - Uses WHERE field > value instead of OFFSET for better performance
  - LIMIT +1 strategy for efficient has_next detection
  - 3 comprehensive cursor encoding/decoding tests

#### Optional COUNT() Queries
- **Performance Optimization** - Skip expensive COUNT queries when not needed:
  - Added `.disable_total_count()` builder method
  - `total` and `total_pages` fields now `Option<u32>` in response metadata
  - Still provides `has_next` via LIMIT +1 strategy without counting
  - Implemented across all database integrations (SQLx, SeaORM, SurrealDB)
  - Web framework integrations conditionally include headers

#### CTE Support
- **Common Table Expression Handling** - Enhanced SQL query support:
  - Automatic detection of WITH clauses in `paginator-sqlx`
  - Proper query wrapping for CTEs with filters and search
  - Prevents breaking complex queries with subquery wrapping
  - Works seamlessly with existing filter and search features

### Security

#### SQL Injection Prevention
- **Critical Security Fix** - Replaced string concatenation with parameterized queries:
  - Created `QueryBuilderExt` trait in `paginator-sqlx/src/query_builder.rs`
  - All filter and search parameters now use `.push_bind()` instead of string interpolation
  - Affects Postgres, MySQL, and SQLite implementations
  - Prevents SQL injection attacks through filter values
  - Comprehensive parameterization for all filter operators (Eq, Ne, Gt, Lt, In, Like, etc.)

### Fixed

#### GitHub Issue #2 - paginator-sqlx 0.2.0 Broken
- **Executor Move Error (E0382)** - Fixed compilation errors in published crate:
  - Properly using `executor.clone()` for intermediate COUNT queries
  - Original executor used for final data query
  - Affects all SQLx database implementations
  - Version bumped to 0.2.1 to replace broken 0.2.0 on crates.io

#### Compilation Warnings
- **Zero Warnings Achievement** - Cleaned up entire codebase:
  - Removed unused `Cursor` import from `paginator-sqlx/src/postgres.rs`
  - Removed unused `Cursor` import from `paginator-sqlx/src/mysql.rs`
  - Removed unused `Cursor` import from `paginator-sqlx/src/sqlite.rs`
  - Removed unused `Cursor` import from `paginator-sea-orm/src/lib.rs`
  - Removed unused `Cursor` import from `paginator-surrealdb/src/query.rs`
  - All 36 tests passing with zero warnings

### Changed

#### Breaking Changes
- **BREAKING**: `PaginatorResponseMeta` fields now optional:
  - `total: u32` → `total: Option<u32>`
  - `total_pages: u32` → `total_pages: Option<u32>`
  - Migration: Use pattern matching or `.unwrap_or()` when accessing these fields
  - Web framework integrations (Axum, Rocket, Actix) updated to handle optional headers
  - Headers only included when values are present

- **Version Bump** - All workspace crates updated from 0.2.0 to 0.2.1:
  - `paginator-utils`: 0.2.1
  - `paginator-rs`: 0.2.1
  - `paginator-sqlx`: 0.2.1
  - `paginator-sea-orm`: 0.2.1
  - `paginator-surrealdb`: 0.2.1
  - `paginator-axum`: 0.2.1
  - `paginator-rocket`: 0.2.1
  - `paginator-actix`: 0.2.1

### Technical Details

#### Cursor Pagination Benefits
- Better performance for large datasets (no OFFSET overhead)
- Consistent results even with concurrent data modifications
- No skipped or duplicate rows during pagination
- Efficient has_next detection without COUNT query

#### Implementation Notes
- Cursor pagination is opt-in and backward compatible
- Operator selection based on sort direction: After+Asc uses >, After+Desc uses <
- Before cursor reverses the operator logic
- Works seamlessly with existing filters and search

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

[0.2.1]: https://github.com/maulanasdqn/paginator-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/maulanasdqn/paginator-rs/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/maulanasdqn/paginator-rs/compare/v0.1.0...v0.1.2
[0.1.0]: https://github.com/maulanasdqn/paginator-rs/releases/tag/v0.1.0
