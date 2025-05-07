#[cfg(test)]
pub mod tests {
    use crate::users_repository::UsersData;
    use paginator::PaginatorTrait;
    use serde_json::json;

    #[test]
    fn test_paginate_struct_output() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let result = users.paginate();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.meta.total, 2);
        assert_eq!(result.data[0].name, "John Doe");
    }

    #[test]
    fn test_paginate_json_output() {
        let users = vec![
            UsersData::new(1, "John Doe".into(), "john@doe.com".into()),
            UsersData::new(2, "Jane Doe".into(), "jane@doe.com".into()),
        ];

        let result = users.paginate_json();
        let expected = json!({
            "data": [
                { "id": 1, "name": "John Doe", "email": "john@doe.com" },
                { "id": 2, "name": "Jane Doe", "email": "jane@doe.com" }
            ],
            "meta": {
                "page": 1,
                "per_page": 2,
                "total": 2
            }
        });

        assert_eq!(result, expected);
    }
}
