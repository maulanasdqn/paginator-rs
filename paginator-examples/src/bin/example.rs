use paginator_examples::users_repository::UsersData;
use paginator_rs::{PaginationParams, PaginatorBuilder, PaginatorTrait};

fn main() {
    let users = vec![
        UsersData::new(1, "John Doe".to_string(), "john@doe.com".to_string()),
        UsersData::new(2, "Jane Doe".to_string(), "jane@doe.com".to_string()),
        UsersData::new(3, "Bob Doe".to_string(), "bob@doe.com".to_string()),
        UsersData::new(4, "Alice Smith".to_string(), "alice@smith.com".to_string()),
        UsersData::new(5, "Charlie Brown".to_string(), "charlie@brown.com".to_string()),
    ];

    println!("=== Example 1: Simple pagination (page 1, 2 items per page) ===");
    let params = PaginationParams::new(1, 2);
    match users.paginate(&params) {
        Ok(result) => println!("{:#?}\n", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 2: Page 2 with 2 items per page ===");
    let params = PaginationParams::new(2, 2);
    match users.paginate(&params) {
        Ok(result) => println!("{:#?}\n", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 3: Using builder pattern with sorting ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(3)
        .sort_by("name")
        .sort_asc()
        .build();
    match users.paginate(&params) {
        Ok(result) => println!("{:#?}\n", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 4: Sort by email descending ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .sort_by("email")
        .sort_desc()
        .build();
    match users.paginate(&params) {
        Ok(result) => println!("{:#?}\n", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 5: JSON output ===");
    let params = PaginationParams::default();
    match users.paginate_json(&params) {
        Ok(json) => println!("{}\n", serde_json::to_string_pretty(&json).unwrap()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
