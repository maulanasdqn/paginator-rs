use paginator_examples::users_repository::UsersData;
use paginator_rs::{FilterValue, PaginatorBuilder, PaginatorTrait};

fn main() {
    let users = vec![
        UsersData::new(1, "John Doe".to_string(), "john@doe.com".to_string()),
        UsersData::new(2, "Jane Doe".to_string(), "jane@doe.com".to_string()),
        UsersData::new(3, "Bob Smith".to_string(), "bob@smith.com".to_string()),
        UsersData::new(4, "Alice Johnson".to_string(), "alice@johnson.com".to_string()),
        UsersData::new(5, "Charlie Brown".to_string(), "charlie@brown.com".to_string()),
        UsersData::new(6, "David Wilson".to_string(), "david@wilson.com".to_string()),
        UsersData::new(7, "Eve Davis".to_string(), "eve@davis.com".to_string()),
        UsersData::new(8, "Frank Miller".to_string(), "frank@miller.com".to_string()),
    ];

    println!("=== Example 1: Filter by ID greater than 3 ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .filter_gt("id", FilterValue::Int(3))
        .build();

    match users.paginate(&params) {
        Ok(result) => {
            println!("Found {} users with ID > 3", result.data.len());
            for user in &result.data {
                println!("  - ID: {}, Name: {}", user.id, user.name);
            }
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 2: Filter with multiple conditions ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .filter_gte("id", FilterValue::Int(2))
        .filter_lte("id", FilterValue::Int(6))
        .sort_by("name")
        .sort_asc()
        .build();

    match users.paginate(&params) {
        Ok(result) => {
            println!("Found {} users with ID between 2 and 6:", result.data.len());
            for user in &result.data {
                println!("  - ID: {}, Name: {}", user.id, user.name);
            }
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 3: Search by name ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .search("john", vec!["name".to_string(), "email".to_string()])
        .build();

    match users.paginate(&params) {
        Ok(result) => {
            println!("Search results for 'john':");
            for user in &result.data {
                println!("  - Name: {}, Email: {}", user.name, user.email);
            }
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 4: Combined filters and search ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(5)
        .filter_gt("id", FilterValue::Int(2))
        .search("smith", vec!["name".to_string(), "email".to_string()])
        .sort_by("id")
        .sort_desc()
        .build();

    match users.paginate(&params) {
        Ok(result) => {
            println!("Combined filter and search results:");
            println!("Total matching: {}", result.meta.total);
            println!("Page {}/{}", result.meta.page, result.meta.total_pages);
            for user in &result.data {
                println!("  - ID: {}, Name: {}, Email: {}", user.id, user.name, user.email);
            }
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 5: Filter using IN operator ===");
    let params = PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .filter_in(
            "id",
            vec![
                FilterValue::Int(1),
                FilterValue::Int(3),
                FilterValue::Int(5),
                FilterValue::Int(7),
            ],
        )
        .build();

    match users.paginate(&params) {
        Ok(result) => {
            println!("Users with ID in [1, 3, 5, 7]:");
            for user in &result.data {
                println!("  - ID: {}, Name: {}", user.id, user.name);
            }
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("=== Example 6: Display SQL WHERE clause ===");
    let params = PaginatorBuilder::new()
        .filter_eq("status", FilterValue::String("active".to_string()))
        .filter_gt("age", FilterValue::Int(18))
        .search("john", vec!["name".to_string(), "email".to_string()])
        .build();

    if let Some(where_clause) = params.to_sql_where() {
        println!("Generated SQL WHERE clause:");
        println!("  WHERE {}", where_clause);
        println!();
    }

    println!("=== Example 7: BETWEEN filter ===");
    let params = PaginatorBuilder::new()
        .filter_between("id", FilterValue::Int(3), FilterValue::Int(6))
        .build();

    match users.paginate(&params) {
        Ok(result) => {
            println!("Users with ID BETWEEN 3 AND 6:");
            for user in &result.data {
                println!("  - ID: {}, Name: {}", user.id, user.name);
            }
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
