//! Keyset (cursor) and relative cursor pagination against in-memory SQLite via SQLx.

use paginator_rs::{
    Cursor, CursorDirection, CursorValue, FilterValue, PaginationParams, PaginatorBuilder,
};
use paginator_sqlx::sqlite::paginate_query;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow, serde::Serialize, Debug)]
struct Item {
    id: i64,
    name: String,
    score: f64,
    uid: String,
}

#[derive(sqlx::FromRow, serde::Serialize, Debug)]
struct Named {
    name: String,
}

async fn pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE items (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            score REAL NOT NULL,
            uid TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    for i in 1..=10i64 {
        sqlx::query("INSERT INTO items (id, name, score, uid) VALUES (?, ?, ?, ?)")
            .bind(i)
            .bind(format!("item{:02}", i))
            .bind(i as f64 * 1.5)
            .bind(format!("00000000-0000-0000-0000-{:012}", i))
            .execute(&pool)
            .await
            .unwrap();
    }
    pool
}

fn ids(items: &[Item]) -> Vec<i64> {
    items.iter().map(|i| i.id).collect()
}

fn decode(encoded: &Option<String>) -> Cursor {
    Cursor::decode(encoded.as_deref().expect("cursor present")).unwrap()
}

async fn run(
    pool: &SqlitePool,
    params: &PaginationParams,
) -> paginator_rs::PaginatorResponse<Item> {
    paginate_query::<_, Item>(pool, "SELECT * FROM items", params)
        .await
        .unwrap()
}

#[tokio::test]
async fn walks_forward_from_an_offset_page() {
    let pool = pool().await;
    let params = PaginatorBuilder::new().per_page(3).sort_by("id").build();
    let first = run(&pool, &params).await.with_cursors("id");
    assert_eq!(ids(&first.data), vec![1, 2, 3]);
    assert_eq!(first.meta.total, Some(10));
    assert!(first.meta.prev_cursor.is_none());

    let mut pages = vec![ids(&first.data)];
    let mut next = first.meta.next_cursor.clone();
    while let Some(cursor) = next {
        let params = PaginatorBuilder::new()
            .per_page(3)
            .sort_by("id")
            .cursor_from_encoded(&cursor)
            .unwrap()
            .disable_total_count()
            .build();
        let page = run(&pool, &params).await;
        assert!(page.meta.has_prev);
        assert!(page.meta.prev_cursor.is_some());
        assert!(page.meta.total.is_none());
        pages.push(ids(&page.data));
        next = page.meta.next_cursor.clone();
    }
    assert_eq!(
        pages,
        vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9], vec![10]]
    );
}

#[tokio::test]
async fn walks_backward_with_prev_cursor() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .cursor_before("id", CursorValue::Int(11))
        .build();
    let last = run(&pool, &params).await;
    assert_eq!(ids(&last.data), vec![8, 9, 10]);
    assert!(
        last.meta.has_next,
        "a Before page always reports rows after it"
    );

    let mut pages = vec![ids(&last.data)];
    let mut prev = last.meta.prev_cursor.clone();
    while let Some(cursor) = prev {
        let params = PaginatorBuilder::new()
            .per_page(3)
            .cursor_from_encoded(&cursor)
            .unwrap()
            .build();
        let page = run(&pool, &params).await;
        pages.push(ids(&page.data));
        prev = page.meta.prev_cursor.clone();
        if prev.is_none() {
            assert!(!page.meta.has_prev);
        }
    }
    assert_eq!(
        pages,
        vec![vec![8, 9, 10], vec![5, 6, 7], vec![2, 3, 4], vec![1]]
    );
}

#[tokio::test]
async fn before_cursor_returns_the_rows_nearest_the_cursor() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .sort_by("id")
        .cursor_before("id", CursorValue::Int(7))
        .build();
    let page = run(&pool, &params).await;
    assert_eq!(ids(&page.data), vec![4, 5, 6]);
    assert!(page.meta.has_prev);
    assert!(page.meta.has_next);

    let next = decode(&page.meta.next_cursor);
    assert_eq!(
        (next.value, next.direction),
        (CursorValue::Int(6), CursorDirection::After)
    );
    let prev = decode(&page.meta.prev_cursor);
    assert_eq!(
        (prev.value, prev.direction),
        (CursorValue::Int(4), CursorDirection::Before)
    );
}

#[tokio::test]
async fn descending_sort_flips_the_comparison() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .sort_by("id")
        .sort_desc()
        .cursor_after("id", CursorValue::Int(8))
        .build();
    let page = run(&pool, &params).await;
    assert_eq!(ids(&page.data), vec![7, 6, 5]);
    assert_eq!(decode(&page.meta.next_cursor).value, CursorValue::Int(5));
    assert_eq!(decode(&page.meta.prev_cursor).value, CursorValue::Int(7));

    let params = PaginatorBuilder::new()
        .per_page(3)
        .sort_by("id")
        .sort_desc()
        .cursor_before("id", CursorValue::Int(3))
        .build();
    let page = run(&pool, &params).await;
    assert_eq!(ids(&page.data), vec![6, 5, 4]);
}

#[tokio::test]
async fn relative_cursor_pagination_skips_pages_from_the_cursor() {
    let pool = pool().await;
    let base = || {
        PaginatorBuilder::new()
            .per_page(3)
            .cursor_after("id", CursorValue::Int(2))
    };
    assert_eq!(
        ids(&run(&pool, &base().page(1).build()).await.data),
        vec![3, 4, 5]
    );

    let page2 = run(&pool, &base().page(2).build()).await;
    assert_eq!(ids(&page2.data), vec![6, 7, 8]);
    assert_eq!(page2.meta.page, 2);
    assert!(page2.meta.has_next && page2.meta.has_prev);
    assert_eq!(decode(&page2.meta.next_cursor).value, CursorValue::Int(8));
    assert_eq!(decode(&page2.meta.prev_cursor).value, CursorValue::Int(6));

    let page3 = run(&pool, &base().page(3).build()).await;
    assert_eq!(ids(&page3.data), vec![9, 10]);
    assert!(!page3.meta.has_next);
    assert!(page3.meta.next_cursor.is_none());

    // Relative paging backwards: the second page of rows before id 9.
    let params = PaginatorBuilder::new()
        .per_page(3)
        .page(2)
        .cursor_before("id", CursorValue::Int(9))
        .build();
    assert_eq!(ids(&run(&pool, &params).await.data), vec![3, 4, 5]);
}

#[tokio::test]
async fn base_query_with_its_own_where_clause() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .cursor_after("id", CursorValue::Int(4))
        .build();
    let page = paginate_query::<_, Item>(&pool, "SELECT * FROM items WHERE id > 2;", &params)
        .await
        .unwrap();
    assert_eq!(ids(&page.data), vec![5, 6, 7]);
    assert_eq!(
        page.meta.total,
        Some(8),
        "total counts the base query, not rows past the cursor"
    );
}

#[tokio::test]
async fn cte_base_query_with_filter_and_cursor() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .filter_gt("score", FilterValue::Float(3.0))
        .cursor_after("id", CursorValue::Int(3))
        .build();
    let page = paginate_query::<_, Item>(
        &pool,
        "WITH big AS (SELECT * FROM items WHERE id > 1) SELECT * FROM big",
        &params,
    )
    .await
    .unwrap();
    assert_eq!(ids(&page.data), vec![4, 5, 6]);
    assert_eq!(page.meta.total, Some(8));
}

#[tokio::test]
async fn cte_base_query_with_filter_and_no_cursor() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .page(2)
        .sort_by("id")
        .filter_gt("id", FilterValue::Int(2))
        .build();
    let page = paginate_query::<_, Item>(
        &pool,
        "WITH big AS (SELECT * FROM items WHERE id > 1) SELECT * FROM big",
        &params,
    )
    .await
    .unwrap();
    assert_eq!(ids(&page.data), vec![6, 7, 8]);
    assert_eq!(page.meta.total, Some(8));
    assert_eq!(page.meta.total_pages, Some(3));
}

#[tokio::test]
async fn sort_by_must_match_the_cursor_field() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .sort_by("name")
        .cursor_after("id", CursorValue::Int(1))
        .build();
    let err = paginate_query::<_, Item>(&pool, "SELECT * FROM items", &params)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("does not match"), "{err}");
}

#[tokio::test]
async fn cursor_without_sort_by_orders_by_the_cursor_field() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .cursor_after("id", CursorValue::Int(5))
        .build();
    assert_eq!(ids(&run(&pool, &params).await.data), vec![6, 7, 8]);
}

#[tokio::test]
async fn string_float_and_uuid_cursor_values() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(2)
        .cursor_after("name", CursorValue::String("item03".into()))
        .build();
    let page = run(&pool, &params).await;
    assert_eq!(ids(&page.data), vec![4, 5]);
    assert_eq!(
        decode(&page.meta.next_cursor).value,
        CursorValue::String("item05".into())
    );

    let params = PaginatorBuilder::new()
        .per_page(2)
        .cursor_after("score", CursorValue::Float(4.5))
        .build();
    let page = run(&pool, &params).await;
    assert_eq!(ids(&page.data), vec![4, 5]);
    assert_eq!(
        decode(&page.meta.next_cursor).value,
        CursorValue::Float(7.5)
    );

    let params = PaginatorBuilder::new()
        .per_page(2)
        .cursor_after(
            "uid",
            CursorValue::Uuid("00000000-0000-0000-0000-000000000004".into()),
        )
        .build();
    let page = run(&pool, &params).await;
    assert_eq!(ids(&page.data), vec![5, 6]);
    assert_eq!(
        decode(&page.meta.next_cursor).value,
        CursorValue::Uuid("00000000-0000-0000-0000-000000000006".into())
    );
}

#[tokio::test]
async fn empty_page_past_the_last_row() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(3)
        .cursor_after("id", CursorValue::Int(10))
        .build();
    let page = run(&pool, &params).await;
    assert!(page.data.is_empty());
    assert!(!page.meta.has_next);
    assert!(page.meta.has_prev);
    assert!(page.meta.next_cursor.is_none());
    assert!(page.meta.prev_cursor.is_none());
    assert_eq!(page.meta.total, Some(10));
}

#[tokio::test]
async fn cursor_field_missing_from_the_row_type_leaves_cursors_unset() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(2)
        .cursor_after("id", CursorValue::Int(3))
        .build();
    let page = paginate_query::<_, Named>(&pool, "SELECT id, name FROM items", &params)
        .await
        .unwrap();
    assert_eq!(
        page.data
            .iter()
            .map(|n| n.name.as_str())
            .collect::<Vec<_>>(),
        vec!["item04", "item05"]
    );
    assert!(page.meta.has_next);
    assert!(page.meta.next_cursor.is_none());
    assert!(page.meta.prev_cursor.is_none());
}

#[tokio::test]
async fn offset_mode_without_count_still_detects_next_page() {
    let pool = pool().await;
    let params = PaginatorBuilder::new()
        .per_page(4)
        .page(3)
        .sort_by("id")
        .disable_total_count()
        .build();
    let page = run(&pool, &params).await;
    assert_eq!(ids(&page.data), vec![9, 10]);
    assert!(!page.meta.has_next);
    assert!(page.meta.has_prev);

    let params = PaginatorBuilder::new()
        .per_page(4)
        .page(2)
        .sort_by("id")
        .disable_total_count()
        .build();
    let page = run(&pool, &params).await;
    assert_eq!(ids(&page.data), vec![5, 6, 7, 8]);
    assert!(page.meta.has_next);
}
