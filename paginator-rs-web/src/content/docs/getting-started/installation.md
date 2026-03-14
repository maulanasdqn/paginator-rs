---
title: Installation
description: How to install paginator-rs and its integrations
---

## Core Library

```toml
[dependencies]
paginator-rs = "0.2.2"
paginator-utils = "0.2.2"
serde = { version = "1", features = ["derive"] }
```

## Database Integrations

### SQLx (PostgreSQL)

```toml
[dependencies]
paginator-sqlx = { version = "0.2.2", features = ["postgres", "runtime-tokio"] }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }
```

### SQLx (MySQL)

```toml
[dependencies]
paginator-sqlx = { version = "0.2.2", features = ["mysql", "runtime-tokio"] }
sqlx = { version = "0.8", features = ["mysql", "runtime-tokio"] }
```

### SQLx (SQLite)

```toml
[dependencies]
paginator-sqlx = { version = "0.2.2", features = ["sqlite", "runtime-tokio"] }
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
```

### SeaORM

```toml
[dependencies]
paginator-sea-orm = { version = "0.2.2", features = ["sqlx-postgres", "runtime-tokio"] }
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio"] }
```

### SurrealDB

```toml
[dependencies]
paginator-surrealdb = { version = "0.2.2", features = ["protocol-ws", "kv-mem"] }
surrealdb = { version = "2.1", features = ["protocol-ws", "kv-mem"] }
```

## Web Framework Integrations

### Axum

```toml
[dependencies]
paginator-axum = "0.2.2"
axum = "0.7"
```

### Rocket

```toml
[dependencies]
paginator-rocket = "0.2.2"
rocket = { version = "0.5", features = ["json"] }
```

### Actix-web

```toml
[dependencies]
paginator-actix = "0.2.2"
actix-web = "4"
```

## Workspace Structure

```
paginator-rs/
├── paginator-rs/         # Core trait and types
├── paginator-utils/      # Shared types (params, response, metadata)
├── paginator-sqlx/       # SQLx database integration
├── paginator-sea-orm/    # SeaORM integration
├── paginator-surrealdb/  # SurrealDB integration
├── paginator-axum/       # Axum web framework integration
├── paginator-rocket/     # Rocket web framework integration
├── paginator-actix/      # Actix-web integration
└── paginator-examples/   # Usage examples
```
