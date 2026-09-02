//! Cursor (keyset) pagination: every `CursorValue` type, both directions,
//! encoding/decoding, and walking pages over an in-memory dataset.
//!
//! Run with: cargo run -p paginator-examples --bin cursors

use paginator_rs::{Cursor, CursorDirection, CursorValue, PaginatorBuilder};

fn main() {
    println!("=== Every CursorValue type ===");
    let cursors = vec![
        Cursor::new("id".into(), CursorValue::Int(42), CursorDirection::After),
        Cursor::new(
            "created_at".into(),
            CursorValue::String("2024-01-01T00:00:00Z".into()),
            CursorDirection::After,
        ),
        Cursor::new(
            "score".into(),
            CursorValue::Float(99.5),
            CursorDirection::Before,
        ),
        Cursor::new(
            "uuid".into(),
            CursorValue::Uuid("550e8400-e29b-41d4-a716-446655440000".into()),
            CursorDirection::After,
        ),
    ];
    for c in &cursors {
        println!("{:?}", c);
    }

    println!("\n=== Encode / decode round-trip ===");
    let cursor = Cursor::new("id".into(), CursorValue::Int(42), CursorDirection::After);
    let encoded = cursor.encode().unwrap();
    println!("encoded: {}", encoded);
    let decoded = Cursor::decode(&encoded).unwrap();
    println!("decoded: {:?}", decoded);

    println!("\n=== Tampered cursor is rejected ===");
    match Cursor::decode("not-a-valid-cursor") {
        Ok(_) => unreachable!(),
        Err(e) => println!("error: {}", e),
    }

    println!("\n=== Builder: after / before / from_encoded ===");
    let after = PaginatorBuilder::new()
        .per_page(20)
        .sort_by("id")
        .cursor_after("id", CursorValue::Int(42))
        .build();
    println!("after:  {:?}", after.cursor);

    let before = PaginatorBuilder::new()
        .per_page(20)
        .sort_by("id")
        .cursor_before("id", CursorValue::Int(42))
        .build();
    println!("before: {:?}", before.cursor);

    let resumed = PaginatorBuilder::new()
        .per_page(20)
        .cursor_from_encoded(&encoded)
        .unwrap()
        .build();
    println!("from_encoded: {:?}", resumed.cursor);

    println!("\n=== Skipping COUNT(*) for performance ===");
    let params = PaginatorBuilder::new()
        .per_page(20)
        .sort_by("created_at")
        .cursor_after(
            "created_at",
            CursorValue::String("2024-01-01T00:00:00Z".into()),
        )
        .disable_total_count()
        .build();
    println!("disable_total_count={}", params.disable_total_count);

    println!("\n=== Walking pages with a cursor (in-memory simulation) ===");
    let ids: Vec<i64> = (1..=10).collect();
    let per_page = 3;
    let mut last_seen: Option<i64> = None;
    loop {
        let page: Vec<i64> = ids
            .iter()
            .filter(|&&id| last_seen.is_none_or(|last| id > last))
            .take(per_page)
            .copied()
            .collect();
        if page.is_empty() {
            break;
        }
        let next = Cursor::new(
            "id".into(),
            CursorValue::Int(*page.last().unwrap()),
            CursorDirection::After,
        );
        println!("page {:?} next_cursor={}", page, next.encode().unwrap());
        last_seen = Some(*page.last().unwrap());
    }
}
