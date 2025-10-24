use paginator_surrealdb::QueryBuilder;

#[test]
fn test_query_builder() {
    let builder = QueryBuilder::new()
        .select("id, name")
        .from("users")
        .where_clause("age > 18")
        .and("status = 'active'");

    let query = builder.build_query().unwrap();
    assert_eq!(
        query,
        "SELECT id, name FROM users WHERE age > 18 AND status = 'active'"
    );
}

#[test]
fn test_query_builder_no_conditions() {
    let builder = QueryBuilder::new().select("*").from("users");

    let query = builder.build_query().unwrap();
    assert_eq!(query, "SELECT * FROM users");
}

#[test]
fn test_query_builder_no_from() {
    let builder = QueryBuilder::new().select("*");

    let result = builder.build_query();
    assert!(result.is_err());
}
