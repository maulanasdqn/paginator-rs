//! Keyset (cursor) and relative cursor pagination against in-memory SQLite via SeaORM.

use paginator_rs::{Cursor, CursorDirection, CursorValue, PaginatorBuilder, SortDirection};
use paginator_sea_orm::{paginate_with_sort, PaginateSeaOrm};
use sea_orm::{
    entity::prelude::*, ActiveValue::Set, Database, DatabaseConnection, QueryOrder, Schema,
};

mod item {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
    #[sea_orm(table_name = "items")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

async fn db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let schema = Schema::new(db.get_database_backend());
    db.execute(&schema.create_table_from_entity(item::Entity))
        .await
        .unwrap();
    for i in 1..=10 {
        item::Entity::insert(item::ActiveModel {
            id: Set(i),
            name: Set(format!("item{:02}", i)),
        })
        .exec(&db)
        .await
        .unwrap();
    }
    db
}

fn ids(items: &[item::Model]) -> Vec<i32> {
    items.iter().map(|i| i.id).collect()
}

fn decode(encoded: &Option<String>) -> Cursor {
    Cursor::decode(encoded.as_deref().expect("cursor present")).unwrap()
}

#[tokio::test]
async fn walks_forward_and_backward() {
    let db = db().await;
    let params = PaginatorBuilder::new().per_page(4).build();
    let first = item::Entity::find()
        .order_by_asc(item::Column::Id)
        .paginate_with(&db, &params)
        .await
        .unwrap()
        .with_cursors("id");
    assert_eq!(ids(&first.data), vec![1, 2, 3, 4]);

    let mut pages = vec![ids(&first.data)];
    let mut next = first.meta.next_cursor.clone();
    while let Some(cursor) = next {
        let params = PaginatorBuilder::new()
            .per_page(4)
            .cursor_from_encoded(&cursor)
            .unwrap()
            .build();
        let page = item::Entity::find()
            .paginate_with(&db, &params)
            .await
            .unwrap();
        assert_eq!(
            page.meta.total,
            Some(10),
            "total is not restricted by the cursor"
        );
        pages.push(ids(&page.data));
        next = page.meta.next_cursor.clone();
    }
    assert_eq!(pages, vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10]]);

    // And back again from the last page's prev_cursor.
    let params = PaginatorBuilder::new()
        .per_page(4)
        .cursor_before("id", CursorValue::Int(9))
        .build();
    let page = item::Entity::find()
        .paginate_with(&db, &params)
        .await
        .unwrap();
    assert_eq!(ids(&page.data), vec![5, 6, 7, 8]);
    assert!(page.meta.has_prev && page.meta.has_next);
    let prev = decode(&page.meta.prev_cursor);
    assert_eq!(
        (prev.value, prev.direction),
        (CursorValue::Int(5), CursorDirection::Before)
    );

    let params = PaginatorBuilder::new()
        .per_page(4)
        .cursor_before("id", CursorValue::Int(5))
        .build();
    let page = item::Entity::find()
        .paginate_with(&db, &params)
        .await
        .unwrap();
    assert_eq!(ids(&page.data), vec![1, 2, 3, 4]);
    assert!(!page.meta.has_prev);
    assert!(page.meta.prev_cursor.is_none());
}

#[tokio::test]
async fn descending_and_relative_pages() {
    let db = db().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .sort_by("id")
        .sort_desc()
        .cursor_after("id", CursorValue::Int(8))
        .build();
    let page = item::Entity::find()
        .paginate_with(&db, &params)
        .await
        .unwrap();
    assert_eq!(ids(&page.data), vec![7, 6, 5]);

    let params = PaginatorBuilder::new()
        .per_page(3)
        .page(2)
        .cursor_after("id", CursorValue::Int(2))
        .build();
    let page = item::Entity::find()
        .paginate_with(&db, &params)
        .await
        .unwrap();
    assert_eq!(ids(&page.data), vec![6, 7, 8]);
    assert_eq!(page.meta.page, 2);
    assert_eq!(decode(&page.meta.next_cursor).value, CursorValue::Int(8));
}

#[tokio::test]
async fn paginate_with_sort_lets_the_cursor_drive_ordering() {
    let db = db().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .sort_by("id")
        .sort_desc()
        .cursor_before("id", CursorValue::Int(4))
        .build();
    let page = paginate_with_sort(item::Entity::find(), &db, &params, |q, _, dir| match dir {
        SortDirection::Asc => q.order_by_asc(item::Column::Id),
        SortDirection::Desc => q.order_by_desc(item::Column::Id),
    })
    .await
    .unwrap();
    assert_eq!(ids(&page.data), vec![7, 6, 5]);
}

#[tokio::test]
async fn mismatched_sort_field_is_rejected() {
    let db = db().await;
    let params = PaginatorBuilder::new()
        .sort_by("name")
        .cursor_after("id", CursorValue::Int(1))
        .build();
    let err = item::Entity::find()
        .paginate_with(&db, &params)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("does not match"), "{err}");
}

#[tokio::test]
async fn offset_mode_without_count_detects_next_page() {
    let db = db().await;
    let params = PaginatorBuilder::new()
        .per_page(4)
        .page(2)
        .disable_total_count()
        .build();
    let page = item::Entity::find()
        .order_by_asc(item::Column::Id)
        .paginate_with(&db, &params)
        .await
        .unwrap();
    assert_eq!(ids(&page.data), vec![5, 6, 7, 8]);
    assert!(page.meta.has_next);

    let params = PaginatorBuilder::new()
        .per_page(4)
        .page(3)
        .disable_total_count()
        .build();
    let page = item::Entity::find()
        .order_by_asc(item::Column::Id)
        .paginate_with(&db, &params)
        .await
        .unwrap();
    assert_eq!(ids(&page.data), vec![9, 10]);
    assert!(!page.meta.has_next);
}
