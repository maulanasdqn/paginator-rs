//! Actix-web integration: query extractor and paginated JSON responses.
//!
//! Run with: cargo run -p paginator-examples --bin actix_server
//! Then try:
//!   curl 'http://localhost:8080/users?page=1&per_page=2'
//!   curl 'http://localhost:8080/users?sort_by=name&sort_direction=desc'

use actix_web::{get, web, App, HttpServer};
use paginator_actix::{PaginatedJson, PaginationQuery};
use paginator_examples::users_repository::UsersData;
use paginator_rs::PaginatorTrait;

#[get("/users")]
async fn get_users(query: web::Query<PaginationQuery>) -> PaginatedJson<UsersData> {
    let params = query.into_inner().into_params();
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

    let response = users.paginate(&params).unwrap();
    let total = response.meta.total.unwrap_or(0);
    PaginatedJson::new(response.data, &params, total)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("actix example listening on http://localhost:8080/users");
    HttpServer::new(|| App::new().service(get_users))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
