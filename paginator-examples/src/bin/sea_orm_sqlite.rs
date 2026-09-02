//! SeaORM 2.0 integration with an in-memory SQLite database. No external services needed.
//!
//! Run with: cargo run -p paginator-examples --bin sea_orm_sqlite

use paginator_rs::{CursorValue, FilterValue, PaginatorBuilder};
use paginator_sea_orm::PaginateSeaOrm;
use sea_orm::{entity::prelude::*, ActiveValue::Set, Database, EntityTrait, Schema};

mod user {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
    #[sea_orm(table_name = "users")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub email: String,
        pub age: i32,
        pub active: bool,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::connect("sqlite::memory:").await?;

    // Create the table from the entity definition
    let schema = Schema::new(db.get_database_backend());
    let stmt = schema.create_table_from_entity(user::Entity);
    db.execute(&stmt).await?;

    for (name, email, age, active) in [
        ("John Doe", "john@doe.com", 28, true),
        ("Jane Doe", "jane@doe.com", 34, true),
        ("Bob Smith", "bob@smith.com", 17, true),
        ("Alice Brown", "alice@brown.com", 45, false),
        ("Charlie Davis", "charlie@davis.com", 52, true),
    ] {
        user::Entity::insert(user::ActiveModel {
            name: Set(name.to_string()),
            email: Set(email.to_string()),
            age: Set(age),
            active: Set(active),
            ..Default::default()
        })
        .exec(&db)
        .await?;
    }

    println!("=== Basic pagination via paginate_with ===");
    let params = PaginatorBuilder::new().page(1).per_page(2).build();
    let result = user::Entity::find().paginate_with(&db, &params).await?;
    println!(
        "page {}/{}: {:?}",
        result.meta.page,
        result.meta.total_pages.unwrap(),
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Filters applied by the paginator ===");
    let params = PaginatorBuilder::new()
        .filter_eq("active", FilterValue::Bool(true))
        .filter_gt("age", FilterValue::Int(18))
        .build();
    let result = user::Entity::find().paginate_with(&db, &params).await?;
    println!(
        "adults, active: {:?}",
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Combining with SeaORM's own query builder ===");
    let params = PaginatorBuilder::new().per_page(10).build();
    let result = user::Entity::find()
        .filter(user::Column::Age.gte(30))
        .paginate_with(&db, &params)
        .await?;
    println!(
        "age >= 30 (SeaORM filter): {:?}",
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Search ===");
    let params = PaginatorBuilder::new()
        .search("doe", vec!["name".into(), "email".into()])
        .build();
    let result = user::Entity::find().paginate_with(&db, &params).await?;
    println!(
        "matching 'doe': {:?}",
        result.data.iter().map(|u| &u.name).collect::<Vec<_>>()
    );

    println!("\n=== Cursor pagination without COUNT(*) ===");
    let params = PaginatorBuilder::new()
        .per_page(2)
        .cursor_after("id", CursorValue::Int(2))
        .disable_total_count()
        .build();
    let result = user::Entity::find().paginate_with(&db, &params).await?;
    println!(
        "after id=2: {:?} has_next={} total={:?}",
        result.data.iter().map(|u| u.id).collect::<Vec<_>>(),
        result.meta.has_next,
        result.meta.total
    );

    println!("\n=== Free function form ===");
    let params = PaginatorBuilder::new().per_page(3).build();
    let result = paginator_sea_orm::paginate(user::Entity::find(), &db, &params).await?;
    println!("first {} of {:?}", result.data.len(), result.meta.total);

    Ok(())
}
