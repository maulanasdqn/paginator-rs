//! Keyset (cursor) and relative cursor pagination against SurrealDB's in-memory engine.

use paginator_rs::{Cursor, CursorDirection, CursorValue, PaginatorBuilder};
use paginator_surrealdb::paginate_query;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::Surreal;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Item {
    n: i64,
    name: String,
}

async fn db() -> Surreal<Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    for n in 1..=10 {
        let _: Option<Item> = db
            .create("items")
            .content(Item {
                n,
                name: format!("item{:02}", n),
            })
            .await
            .unwrap();
    }
    db
}

fn ns(items: &[Item]) -> Vec<i64> {
    items.iter().map(|i| i.n).collect()
}

fn decode(encoded: &Option<String>) -> Cursor {
    Cursor::decode(encoded.as_deref().expect("cursor present")).unwrap()
}

#[tokio::test]
async fn walks_forward_and_backward() {
    let db = db().await;
    let params = PaginatorBuilder::new().per_page(4).sort_by("n").build();
    let first = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
        .await
        .unwrap()
        .with_cursors("n");
    assert_eq!(ns(&first.data), vec![1, 2, 3, 4]);

    let mut pages = vec![ns(&first.data)];
    let mut next = first.meta.next_cursor.clone();
    while let Some(cursor) = next {
        let params = PaginatorBuilder::new()
            .per_page(4)
            .sort_by("n")
            .cursor_from_encoded(&cursor)
            .unwrap()
            .build();
        let page = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
            .await
            .unwrap();
        assert_eq!(page.meta.total, Some(10));
        pages.push(ns(&page.data));
        next = page.meta.next_cursor.clone();
    }
    assert_eq!(pages, vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10]]);

    let params = PaginatorBuilder::new()
        .per_page(4)
        .cursor_before("n", CursorValue::Int(9))
        .build();
    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
        .await
        .unwrap();
    assert_eq!(ns(&page.data), vec![5, 6, 7, 8]);
    let prev = decode(&page.meta.prev_cursor);
    assert_eq!(
        (prev.value, prev.direction),
        (CursorValue::Int(5), CursorDirection::Before)
    );

    let params = PaginatorBuilder::new()
        .per_page(4)
        .cursor_before("n", CursorValue::Int(5))
        .build();
    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
        .await
        .unwrap();
    assert_eq!(ns(&page.data), vec![1, 2, 3, 4]);
    assert!(!page.meta.has_prev);
}

#[tokio::test]
async fn descending_relative_and_where_clause() {
    let db = db().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .sort_by("n")
        .sort_desc()
        .cursor_after("n", CursorValue::Int(8))
        .build();
    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
        .await
        .unwrap();
    assert_eq!(ns(&page.data), vec![7, 6, 5]);

    let params = PaginatorBuilder::new()
        .per_page(3)
        .page(2)
        .cursor_after("n", CursorValue::Int(2))
        .build();
    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
        .await
        .unwrap();
    assert_eq!(ns(&page.data), vec![6, 7, 8]);
    assert_eq!(page.meta.page, 2);

    let params = PaginatorBuilder::new()
        .per_page(3)
        .cursor_after("n", CursorValue::Int(4))
        .build();
    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items WHERE n > 2", &params)
        .await
        .unwrap();
    assert_eq!(ns(&page.data), vec![5, 6, 7]);
    assert_eq!(page.meta.total, Some(8));
}

#[tokio::test]
async fn string_cursor_and_offset_mode_without_count() {
    let db = db().await;
    let params = PaginatorBuilder::new()
        .per_page(2)
        .cursor_after("name", CursorValue::String("item03".into()))
        .build();
    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
        .await
        .unwrap();
    assert_eq!(ns(&page.data), vec![4, 5]);
    assert_eq!(
        decode(&page.meta.next_cursor).value,
        CursorValue::String("item05".into())
    );

    let params = PaginatorBuilder::new()
        .per_page(4)
        .page(3)
        .sort_by("n")
        .disable_total_count()
        .build();
    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
        .await
        .unwrap();
    assert_eq!(ns(&page.data), vec![9, 10]);
    assert!(!page.meta.has_next);
}

#[tokio::test]
async fn total_counts_all_matching_rows() {
    let db = db().await;
    let params = PaginatorBuilder::new().per_page(3).build();
    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items", &params)
        .await
        .unwrap();
    assert_eq!(page.meta.total, Some(10));
    assert_eq!(page.meta.total_pages, Some(4));
    assert!(page.meta.has_next);

    let page = paginate_query::<Item, _>(&db, "SELECT * FROM items WHERE n > 100", &params)
        .await
        .unwrap();
    assert!(page.data.is_empty());
    assert_eq!(page.meta.total, Some(0));
    assert!(!page.meta.has_next);
}
