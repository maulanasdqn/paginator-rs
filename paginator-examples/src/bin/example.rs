use paginator::PaginatorTrait;
use paginator_examples::users_repository::UsersData;

fn main() {
    let users = vec![
        UsersData::new(1, "John Doe".to_string(), "john@doe.com".to_string()),
        UsersData::new(2, "Jane Doe".to_string(), "jane@doe.com".to_string()),
        UsersData::new(3, "Bob Doe".to_string(), "bob@doe.com".to_string()),
    ];
    println!("{:#?}", users.paginate());
    println!("{}", users.paginate_json());
}
