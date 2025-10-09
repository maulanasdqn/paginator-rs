#[cfg(test)]
pub mod tests {
    use crate::users_repository::UsersData;
    use paginator_rs::{FilterValue, PaginationParams, PaginatorBuilder, PaginatorTrait};
    use serde_json::json;

    #[test]
    fn test_paginate_struct_output() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginationParams::new(1, 10);
        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.meta.total, 2);
        assert_eq!(result.meta.total_pages, 1);
        assert_eq!(result.meta.has_next, false);
        assert_eq!(result.meta.has_prev, false);
        assert_eq!(result.data[0].name, "John Doe");
    }

    #[test]
    fn test_paginate_json_output() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginationParams::new(1, 10);
        let result = users.paginate_json(&params).unwrap();
        let expected = json!({
            "data": [
                { "id": 1, "name": "John Doe", "email": "john@doe.com" },
                { "id": 2, "name": "Jane Doe", "email": "jane@doe.com" }
            ],
            "meta": {
                "page": 1,
                "per_page": 10,
                "total": 2,
                "total_pages": 1,
                "has_next": false,
                "has_prev": false
            }
        });

        assert_eq!(result, expected);
    }

    // ==================== PAGINATION EDGE CASES ====================

    #[test]
    fn test_empty_dataset() {
        let users: Vec<UsersData> = vec![];
        let params = PaginationParams::new(1, 10);
        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 0);
        assert_eq!(result.meta.total, 0);
        assert_eq!(result.meta.total_pages, 0);
        assert_eq!(result.meta.has_next, false);
        assert_eq!(result.meta.has_prev, false);
    }

    #[test]
    fn test_single_item() {
        let users = vec![UsersData::new(1, "John Doe".into(), "john@doe.com".into())];
        let params = PaginationParams::new(1, 10);
        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.meta.total_pages, 1);
        assert_eq!(result.meta.has_next, false);
        assert_eq!(result.meta.has_prev, false);
    }

    #[test]
    fn test_exact_page_boundary() {
        let users: Vec<UsersData> = (1..=10)
            .map(|i| UsersData::new(i, format!("User {}", i), format!("user{}@test.com", i)))
            .collect();

        let params = PaginationParams::new(1, 10);
        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 10);
        assert_eq!(result.meta.total, 10);
        assert_eq!(result.meta.total_pages, 1);
        assert_eq!(result.meta.has_next, false);
        assert_eq!(result.meta.has_prev, false);
    }

    #[test]
    fn test_multiple_pages() {
        let users: Vec<UsersData> = (1..=25)
            .map(|i| UsersData::new(i, format!("User {}", i), format!("user{}@test.com", i)))
            .collect();

        // Page 1
        let params = PaginationParams::new(1, 10);
        let result = users.paginate(&params).unwrap();
        assert_eq!(result.data.len(), 10);
        assert_eq!(result.meta.page, 1);
        assert_eq!(result.meta.total, 25);
        assert_eq!(result.meta.total_pages, 3);
        assert_eq!(result.meta.has_next, true);
        assert_eq!(result.meta.has_prev, false);

        // Page 2
        let params = PaginationParams::new(2, 10);
        let result = users.paginate(&params).unwrap();
        assert_eq!(result.data.len(), 10);
        assert_eq!(result.meta.page, 2);
        assert_eq!(result.meta.has_next, true);
        assert_eq!(result.meta.has_prev, true);

        // Page 3 (partial)
        let params = PaginationParams::new(3, 10);
        let result = users.paginate(&params).unwrap();
        assert_eq!(result.data.len(), 5);
        assert_eq!(result.meta.page, 3);
        assert_eq!(result.meta.has_next, false);
        assert_eq!(result.meta.has_prev, true);
    }

    #[test]
    fn test_page_beyond_total() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginationParams::new(10, 10);
        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 0);
        assert_eq!(result.meta.total, 2);
        assert_eq!(result.meta.total_pages, 1);
        assert_eq!(result.meta.page, 10);
    }

    #[test]
    fn test_large_per_page() {
        let users: Vec<UsersData> = (1..=5)
            .map(|i| UsersData::new(i, format!("User {}", i), format!("user{}@test.com", i)))
            .collect();

        let params = PaginationParams::new(1, 100);
        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 5);
        assert_eq!(result.meta.total, 5);
        assert_eq!(result.meta.total_pages, 1);
    }

    // ==================== FILTERING EDGE CASES ====================

    #[test]
    fn test_filter_no_matches() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .page(1)
            .per_page(10)
            .filter_eq("id", FilterValue::Int(999))
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 0);
        assert_eq!(result.meta.total, 0);
    }

    #[test]
    fn test_filter_all_match() {
        let users: Vec<UsersData> = (1..=5)
            .map(|i| UsersData::new(i, format!("User {}", i), format!("user{}@test.com", i)))
            .collect();

        let params = PaginatorBuilder::new()
            .page(1)
            .per_page(10)
            .filter_gt("id", FilterValue::Int(0))
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 5);
        assert_eq!(result.meta.total, 5);
    }

    #[test]
    fn test_multiple_filters_and_logic() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
            UsersData::new(3, "Bob Smith".into(), "bob@smith.com".into()),
            UsersData::new(4, "Alice Johnson".into(), "alice@johnson.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .page(1)
            .per_page(10)
            .filter_gt("id", FilterValue::Int(1))
            .filter_lt("id", FilterValue::Int(4))
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].id, 2);
        assert_eq!(result.data[1].id, 3);
    }

    #[test]
    fn test_filter_eq_operator() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .filter_eq("id", FilterValue::Int(2))
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].id, 2);
        assert_eq!(result.data[0].name, "Jane Doe");
    }

    #[test]
    fn test_filter_ne_operator() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
            UsersData::new(3, "Bob Smith".into(), "bob@smith.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .filter_ne("id", FilterValue::Int(2))
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].id, 1);
        assert_eq!(result.data[1].id, 3);
    }

    #[test]
    fn test_filter_gte_lte_operators() {
        let users: Vec<UsersData> = (1..=10)
            .map(|i| UsersData::new(i, format!("User {}", i), format!("user{}@test.com", i)))
            .collect();

        let params = PaginatorBuilder::new()
            .filter_gte("id", FilterValue::Int(3))
            .filter_lte("id", FilterValue::Int(7))
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 5);
        assert_eq!(result.data[0].id, 3);
        assert_eq!(result.data[4].id, 7);
    }

    #[test]
    fn test_filter_in_operator() {
        let users: Vec<UsersData> = (1..=10)
            .map(|i| UsersData::new(i, format!("User {}", i), format!("user{}@test.com", i)))
            .collect();

        let params = PaginatorBuilder::new()
            .filter_in(
                "id",
                vec![
                    FilterValue::Int(2),
                    FilterValue::Int(5),
                    FilterValue::Int(8),
                ],
            )
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 3);
        assert_eq!(result.data[0].id, 2);
        assert_eq!(result.data[1].id, 5);
        assert_eq!(result.data[2].id, 8);
    }

    #[test]
    fn test_filter_between_operator() {
        let users: Vec<UsersData> = (1..=10)
            .map(|i| UsersData::new(i, format!("User {}", i), format!("user{}@test.com", i)))
            .collect();

        let params = PaginatorBuilder::new()
            .filter_between("id", FilterValue::Int(4), FilterValue::Int(6))
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 3);
        assert_eq!(result.data[0].id, 4);
        assert_eq!(result.data[1].id, 5);
        assert_eq!(result.data[2].id, 6);
    }

    #[test]
    fn test_filter_like_operator() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
            UsersData::new(3, "Bob Smith".into(), "bob@smith.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .filter_like("name", "%Doe%")
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 2);
        assert!(result.data[0].name.contains("Doe"));
        assert!(result.data[1].name.contains("Doe"));
    }

    // ==================== SEARCH EDGE CASES ====================

    #[test]
    fn test_search_no_matches() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .search("NonExistent", vec!["name".to_string()])
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 0);
        assert_eq!(result.meta.total, 0);
    }

    #[test]
    fn test_search_case_insensitive() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .search("JOHN", vec!["name".to_string()])
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].name, "John Doe");
    }

    #[test]
    fn test_search_multiple_fields() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Smith".into(), "jane@doe.com".into()),
            UsersData::new(3, "Bob Wilson".into(), "bob@wilson.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .search("doe", vec!["name".to_string(), "email".to_string()])
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 2);
    }

    #[test]
    fn test_search_partial_match() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
            UsersData::new(3, "Johnny Smith".into(), "johnny@smith.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .search("john", vec!["name".to_string()])
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 2);
    }

    #[test]
    fn test_search_exact_match() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .search_exact("John Doe", vec!["name".to_string()])
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].name, "John Doe");
    }

    #[test]
    fn test_search_case_sensitive() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "jane doe".into(), "jane@doe.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .search_case_sensitive("John", vec!["name".to_string()])
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].name, "John Doe");
    }

    // ==================== SORTING EDGE CASES ====================

    #[test]
    fn test_sort_ascending() {
        let users = vec![
            UsersData::new(3, "Charlie".into(), "charlie@test.com".into()),
            UsersData::new(1, "Alice".into(), "alice@test.com".into()),
            UsersData::new(2, "Bob".into(), "bob@test.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .sort_by("name")
            .sort_asc()
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data[0].name, "Alice");
        assert_eq!(result.data[1].name, "Bob");
        assert_eq!(result.data[2].name, "Charlie");
    }

    #[test]
    fn test_sort_descending() {
        let users = vec![
            UsersData::new(1, "Alice".into(), "alice@test.com".into()),
            UsersData::new(2, "Bob".into(), "bob@test.com".into()),
            UsersData::new(3, "Charlie".into(), "charlie@test.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .sort_by("name")
            .sort_desc()
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data[0].name, "Charlie");
        assert_eq!(result.data[1].name, "Bob");
        assert_eq!(result.data[2].name, "Alice");
    }

    #[test]
    fn test_sort_by_id() {
        let users = vec![
            UsersData::new(5, "User 5".into(), "user5@test.com".into()),
            UsersData::new(2, "User 2".into(), "user2@test.com".into()),
            UsersData::new(8, "User 8".into(), "user8@test.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .sort_by("id")
            .sort_asc()
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data[0].id, 2);
        assert_eq!(result.data[1].id, 5);
        assert_eq!(result.data[2].id, 8);
    }

    // ==================== COMBINED OPERATIONS ====================

    #[test]
    fn test_filter_and_search_combined() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@example.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@example.com".into()),
            UsersData::new(3, "John Smith".into(), "john@smith.com".into()),
            UsersData::new(4, "Bob Johnson".into(), "bob@example.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .filter_like("email", "%example%")
            .search("John", vec!["name".to_string()])
            .build();

        let result = users.paginate(&params).unwrap();

        // Both "John Doe" and "Bob Johnson" match:
        // - Both have emails with "example"
        // - Both have "John" in their name
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].name, "John Doe");
        assert_eq!(result.data[1].name, "Bob Johnson");
    }

    #[test]
    fn test_filter_search_sort_pagination_combined() {
        let users: Vec<UsersData> = vec![
            UsersData::new(1, "Alice Developer".into(), "alice@dev.com".into()),
            UsersData::new(2, "Bob Developer".into(), "bob@dev.com".into()),
            UsersData::new(3, "Charlie Designer".into(), "charlie@design.com".into()),
            UsersData::new(4, "David Developer".into(), "david@dev.com".into()),
            UsersData::new(5, "Eve Designer".into(), "eve@design.com".into()),
            UsersData::new(6, "Frank Developer".into(), "frank@dev.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .page(1)
            .per_page(2)
            .filter_like("email", "%dev.com%")
            .search("Developer", vec!["name".to_string()])
            .sort_by("name")
            .sort_asc()
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.meta.total, 4); // All developers
        assert_eq!(result.meta.total_pages, 2);
        assert_eq!(result.meta.has_next, true);
        assert_eq!(result.data[0].name, "Alice Developer");
        assert_eq!(result.data[1].name, "Bob Developer");

        // Test page 2
        let params_page2 = PaginatorBuilder::new()
            .page(2)
            .per_page(2)
            .filter_like("email", "%dev.com%")
            .search("Developer", vec!["name".to_string()])
            .sort_by("name")
            .sort_asc()
            .build();

        let result_page2 = users.paginate(&params_page2).unwrap();

        assert_eq!(result_page2.data.len(), 2);
        assert_eq!(result_page2.meta.has_next, false);
        assert_eq!(result_page2.meta.has_prev, true);
        assert_eq!(result_page2.data[0].name, "David Developer");
        assert_eq!(result_page2.data[1].name, "Frank Developer");
    }

    #[test]
    fn test_sort_with_filter() {
        let users = vec![
            UsersData::new(5, "Eve".into(), "eve@test.com".into()),
            UsersData::new(2, "Bob".into(), "bob@test.com".into()),
            UsersData::new(7, "Grace".into(), "grace@test.com".into()),
            UsersData::new(3, "Charlie".into(), "charlie@test.com".into()),
            UsersData::new(9, "Ivan".into(), "ivan@test.com".into()),
        ];

        let params = PaginatorBuilder::new()
            .filter_gt("id", FilterValue::Int(3))
            .sort_by("name")
            .sort_asc()
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 3);
        assert_eq!(result.data[0].name, "Eve");
        assert_eq!(result.data[1].name, "Grace");
        assert_eq!(result.data[2].name, "Ivan");
    }

    #[test]
    fn test_large_dataset_pagination() {
        let users: Vec<UsersData> = (1..=100)
            .map(|i| UsersData::new(i, format!("User {}", i), format!("user{}@test.com", i)))
            .collect();

        let params = PaginatorBuilder::new()
            .page(5)
            .per_page(20)
            .build();

        let result = users.paginate(&params).unwrap();

        assert_eq!(result.data.len(), 20);
        assert_eq!(result.meta.total, 100);
        assert_eq!(result.meta.total_pages, 5);
        assert_eq!(result.meta.page, 5);
        assert_eq!(result.meta.has_next, false);
        assert_eq!(result.meta.has_prev, true);
        assert_eq!(result.data[0].id, 81);
        assert_eq!(result.data[19].id, 100);
    }
}
