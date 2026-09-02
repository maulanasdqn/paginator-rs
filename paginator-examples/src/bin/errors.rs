//! Every `PaginatorError` variant and how to handle them.
//!
//! Run with: cargo run -p paginator-examples --bin errors

use paginator_examples::users_repository::UsersData;
use paginator_rs::{PaginationParams, PaginatorError, PaginatorResult, PaginatorTrait};

fn handle<T: std::fmt::Debug>(label: &str, result: PaginatorResult<T>) {
    match result {
        Ok(_) => println!("{label}: ok"),
        Err(PaginatorError::InvalidPage(page)) => {
            println!("{label}: invalid page {page} (must be >= 1)")
        }
        Err(PaginatorError::InvalidPerPage(per_page)) => {
            println!("{label}: invalid per_page {per_page} (must be 1..=100)")
        }
        Err(PaginatorError::SerializationError(msg)) => {
            println!("{label}: serialization failed: {msg}")
        }
        Err(PaginatorError::Custom(msg)) => println!("{label}: {msg}"),
    }
}

fn main() {
    let users = vec![UsersData::new(
        1,
        "John".to_string(),
        "john@doe.com".to_string(),
    )];

    println!("=== InvalidPage ===");
    let params = PaginationParams {
        page: 0,
        ..Default::default()
    };
    handle("page=0", users.paginate(&params));

    println!("\n=== InvalidPerPage ===");
    let params = PaginationParams {
        per_page: 0,
        ..Default::default()
    };
    handle("per_page=0", users.paginate(&params));
    let params = PaginationParams {
        per_page: 101,
        ..Default::default()
    };
    handle("per_page=101", users.paginate(&params));

    println!("\n=== Custom (e.g. database integrations wrap query failures) ===");
    let custom: PaginatorResult<()> = Err(PaginatorError::Custom("Count query failed: ...".into()));
    handle("custom", custom);

    println!("\n=== Errors implement std::error::Error + Display ===");
    let err = PaginatorError::InvalidPage(0);
    println!("Display: {}", err);
    println!("Debug:   {:?}", err);

    println!("\n=== Note: the builder clamps instead of erroring ===");
    let params = paginator_rs::PaginatorBuilder::new()
        .page(0)
        .per_page(9999)
        .build();
    println!(
        "builder page=0 per_page=9999 -> page={} per_page={}",
        params.page, params.per_page
    );
}
