---
title: SeaORM
description: Type-safe ORM pagination with SeaORM entities
---

The `paginator-sea-orm` crate provides type-safe pagination for SeaORM entities.

## Installation

```toml
[dependencies]
paginator-sea-orm = { version = "0.2.2", features = ["sqlx-postgres", "runtime-tokio"] }
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio"] }
```

## Using the Trait

The `PaginateSeaOrm` trait extends SeaORM's `Select` with pagination:

```rust
use paginator_sea_orm::PaginateSeaOrm;
use paginator_rs::PaginationParams;
use sea_orm::EntityTrait;

async fn list_users(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let params = PaginationParams::new(1, 20);

    let result = User::find()
        .filter(user::Column::Active.eq(true))
        .paginate_with(db, &params)
        .await?;

    println!("Found {} users on page {}", result.data.len(), result.meta.page);
    Ok(())
}
```

## Using the Helper Function

```rust
use paginator_sea_orm::paginate;
use paginator_rs::Paginator;

let params = Paginator::new()
    .page(1)
    .per_page(20)
    .sort().desc("created_at")
    .build();

let result = paginate(
    User::find().filter(user::Column::Active.eq(true)),
    db,
    &params,
).await?;
```

## Custom Sort Function

Use `paginate_with_sort` for custom sorting logic:

```rust
use paginator_sea_orm::paginate_with_sort;

let result = paginate_with_sort(
    User::find(),
    db,
    &params,
    |select, sort_by, direction| {
        // Custom sorting logic
        match sort_by {
            "name" => select.order_by(user::Column::Name, direction.into()),
            "email" => select.order_by(user::Column::Email, direction.into()),
            _ => select.order_by(user::Column::Id, direction.into()),
        }
    },
).await?;
```

## Features

- Automatic conversion of `FilterValue` and `CursorValue` to SeaORM values
- Builds SeaORM conditions from pagination filters
- Supports cursor pagination with SeaORM entities
- Type-safe query building
