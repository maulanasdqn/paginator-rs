//! All search modes: fuzzy, exact, case-sensitive, and direct `SearchParams`.
//!
//! Run with: cargo run -p paginator-examples --bin search

use paginator_rs::{PaginatorBuilder, SearchParams};

fn main() {
    println!("=== Fuzzy search (default: case-insensitive, partial match) ===");
    let params = PaginatorBuilder::new()
        .search("john", vec!["name".into(), "email".into()])
        .build();
    let search = params.search.unwrap();
    println!("{:?}", search);
    println!("SQL: {}", search.to_sql_where());

    println!("\n=== Exact match ===");
    let params = PaginatorBuilder::new()
        .search_exact("John Doe", vec!["name".into()])
        .build();
    let search = params.search.unwrap();
    println!("{:?}", search);
    println!("SQL: {}", search.to_sql_where());

    println!("\n=== Case-sensitive ===");
    let params = PaginatorBuilder::new()
        .search_case_sensitive("John", vec!["name".into()])
        .build();
    let search = params.search.unwrap();
    println!("{:?}", search);
    println!("SQL: {}", search.to_sql_where());

    println!("\n=== Direct SearchParams construction ===");
    let search = SearchParams::new("rust developer", vec!["title".into(), "bio".into()])
        .with_case_sensitive(true)
        .with_exact_match(false);
    println!("{:?}", search);
    println!("SQL: {}", search.to_sql_where());

    println!("\n=== Multi-field search combined with pagination ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .search(
            "developer",
            vec!["title".into(), "bio".into(), "skills".into()],
        )
        .sort_by("relevance")
        .sort_desc()
        .build();
    println!("WHERE {}", params.to_sql_where().unwrap());
}
