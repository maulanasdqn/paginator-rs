//! Axum integration: query extractor and paginated JSON responses with headers.
//!
//! Run with: cargo run -p paginator-examples --bin axum_server
//! Then try:
//!   curl 'http://localhost:3000/users?page=1&per_page=2'
//!   curl 'http://localhost:3000/users?filter=age:gt:18&sort_by=name&sort_direction=asc'
//!   curl 'http://localhost:3000/users?search=doe&search_fields=name,email'
//!   curl -i 'http://localhost:3000/users'   # shows X-Total-Count etc.

use axum::{routing::get, Router};
use paginator_axum::{PaginatedJson, PaginationQuery};
use paginator_examples::users_repository::UsersData;
use paginator_rs::PaginatorTrait;

async fn get_users(PaginationQuery(params): PaginationQuery) -> PaginatedJson<UsersData> {
    let users = vec![
        UsersData::new(1, "John Doe".to_string(), "john@doe.com".to_string()),
        UsersData::new(2, "Jane Doe".to_string(), "jane@doe.com".to_string()),
        UsersData::new(3, "Bob Smith".to_string(), "bob@smith.com".to_string()),
        UsersData::new(4, "Alice Brown".to_string(), "alice@brown.com".to_string()),
        UsersData::new(
            5,
            "Charlie Davis".to_string(),
            "charlie@davis.com".to_string(),
        ),
    ];

    // The repository applies filters/search/sort in memory; a real app
    // would push params down to the database instead.
    let response = users.paginate(&params).unwrap();
    let total = response.meta.total.unwrap_or(0);
    PaginatedJson::new(response.data, &params, total)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/users", get(get_users));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("axum example listening on http://localhost:3000/users");
    axum::serve(listener, app).await.unwrap();
}
