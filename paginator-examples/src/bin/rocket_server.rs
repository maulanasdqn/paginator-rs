//! Rocket integration: request guard and paginated JSON responses.
//!
//! Run with: cargo run -p paginator-examples --bin rocket_server
//! Then try:
//!   curl 'http://localhost:8000/api/users?page=1&per_page=2'
//!   curl 'http://localhost:8000/api/users?sort_by=name&sort_direction=asc'

use paginator_examples::users_repository::UsersData;
use paginator_rocket::{PaginatedJson, Pagination};
use paginator_rs::PaginatorTrait;
use rocket::{get, launch, routes};

#[get("/users")]
async fn get_users(pagination: Pagination) -> PaginatedJson<UsersData> {
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

    let response = users.paginate(&pagination.params).unwrap();
    let total = response.meta.total.unwrap_or(0);
    PaginatedJson::new(response.data, &pagination.params, total)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/api", routes![get_users])
}
