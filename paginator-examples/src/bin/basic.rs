//! Basic pagination: `PaginationParams`, `PaginatorTrait`, and response types.
//!
//! Run with: cargo run -p paginator-examples --bin basic

use paginator_examples::users_repository::UsersData;
use paginator_rs::{PaginationParams, PaginatorResponseMeta, PaginatorTrait, SortDirection};

fn sample_users() -> Vec<UsersData> {
    vec![
        UsersData::new(1, "John Doe".to_string(), "john@doe.com".to_string()),
        UsersData::new(2, "Jane Doe".to_string(), "jane@doe.com".to_string()),
        UsersData::new(3, "Bob Doe".to_string(), "bob@doe.com".to_string()),
        UsersData::new(4, "Alice Smith".to_string(), "alice@smith.com".to_string()),
        UsersData::new(
            5,
            "Charlie Brown".to_string(),
            "charlie@brown.com".to_string(),
        ),
    ]
}

fn main() {
    let users = sample_users();

    println!("=== Defaults ===");
    let params = PaginationParams::default();
    println!("page={} per_page={}", params.page, params.per_page);

    println!("\n=== PaginationParams::new ===");
    let params = PaginationParams::new(2, 2);
    println!("offset={} limit={}", params.offset(), params.limit());
    println!("{:#?}", users.paginate(&params).unwrap());

    println!("\n=== Sorting with with_sort / with_direction ===");
    let params = PaginationParams::new(1, 3)
        .with_sort("name")
        .with_direction(SortDirection::Desc);
    println!("{:#?}", users.paginate(&params).unwrap());

    println!("\n=== JSON output via paginate_json ===");
    let params = PaginationParams::new(1, 2);
    let json = users.paginate_json(&params).unwrap();
    println!("{}", serde_json::to_string_pretty(&json).unwrap());

    println!("\n=== Response metadata variants ===");
    // Standard: total known
    let meta = PaginatorResponseMeta::new(1, 20, 100);
    println!("with total:      {}", serde_json::to_string(&meta).unwrap());
    // COUNT(*) skipped: total/total_pages omitted
    let meta = PaginatorResponseMeta::new_without_total(1, 20, true);
    println!("without total:   {}", serde_json::to_string(&meta).unwrap());
    // Cursor pagination: next/prev cursors included
    let meta = PaginatorResponseMeta::new_with_cursors(
        1,
        20,
        None,
        true,
        Some("next-cursor".to_string()),
        Some("prev-cursor".to_string()),
    );
    println!("with cursors:    {}", serde_json::to_string(&meta).unwrap());
}
