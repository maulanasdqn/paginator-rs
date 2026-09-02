use paginator_utils::{FilterOperator, FilterValue, SortDirection};
use paginator_zod::{typescript, PaginationSchema};
use serde_json::json;

#[test]
fn accepts_valid_input_and_normalizes() {
    let schema = PaginationSchema::new();
    let params = schema
        .validate(&json!({
            "page": 3,
            "per_page": 25,
            "sort_by": "name",
            "sort_direction": "asc"
        }))
        .unwrap();

    assert_eq!(params.page, 3);
    assert_eq!(params.per_page, 25);
    assert_eq!(params.sort_by.as_deref(), Some("name"));
    assert_eq!(params.sort_direction, Some(SortDirection::Asc));
}

#[test]
fn applies_defaults_when_omitted() {
    let params = PaginationSchema::new()
        .default_per_page(30)
        .validate(&json!({}))
        .unwrap();
    assert_eq!(params.page, 1);
    assert_eq!(params.per_page, 30);
}

#[test]
fn rejects_per_page_over_max() {
    let err = PaginationSchema::new()
        .max_per_page(50)
        .validate(&json!({ "per_page": 51 }))
        .unwrap_err();
    assert_eq!(err.issues.len(), 1);
    assert_eq!(err.issues[0].path, vec!["per_page"]);
}

#[test]
fn rejects_non_integer_page() {
    let err = PaginationSchema::new()
        .validate(&json!({ "page": 1.5 }))
        .unwrap_err();
    assert_eq!(err.issues[0].path, vec!["page"]);
}

#[test]
fn rejects_bad_sort_direction() {
    let err = PaginationSchema::new()
        .validate(&json!({ "sort_direction": "sideways" }))
        .unwrap_err();
    assert_eq!(err.issues[0].path, vec!["sort_direction"]);
}

#[test]
fn enforces_allowed_sort_fields() {
    let schema = PaginationSchema::new().allowed_sort_fields(["name", "created_at"]);
    assert!(schema.validate(&json!({ "sort_by": "name" })).is_ok());

    let err = schema
        .validate(&json!({ "sort_by": "password" }))
        .unwrap_err();
    assert_eq!(err.issues[0].path, vec!["sort_by"]);
}

#[test]
fn parses_filters_with_operator_and_value() {
    let params = PaginationSchema::new()
        .validate(&json!({
            "filters": [
                { "field": "status", "operator": "eq", "value": "active" },
                { "field": "age", "operator": "gt", "value": 18 }
            ]
        }))
        .unwrap();

    assert_eq!(params.filters.len(), 2);
    assert_eq!(params.filters[0].field, "status");
    assert_eq!(params.filters[0].operator, FilterOperator::Eq);
    assert_eq!(
        params.filters[0].value,
        FilterValue::String("active".into())
    );
    assert_eq!(params.filters[1].operator, FilterOperator::Gt);
    assert_eq!(params.filters[1].value, FilterValue::Int(18));
}

#[test]
fn rejects_invalid_filter_operator_with_nested_path() {
    let err = PaginationSchema::new()
        .validate(&json!({
            "filters": [{ "field": "status", "operator": "drop_table", "value": 1 }]
        }))
        .unwrap_err();
    assert_eq!(err.issues[0].path, vec!["filters", "0", "operator"]);
}

#[test]
fn enforces_allowed_filter_fields() {
    let schema = PaginationSchema::new().allowed_filter_fields(["status"]);
    assert!(schema
        .validate(&json!({ "filters": [{ "field": "status", "operator": "eq", "value": 1 }] }))
        .is_ok());

    let err = schema
        .validate(&json!({ "filters": [{ "field": "secret", "operator": "eq", "value": 1 }] }))
        .unwrap_err();
    assert_eq!(err.issues[0].path, vec!["filters", "0", "field"]);
}

#[test]
fn parses_search() {
    let params = PaginationSchema::new()
        .validate(&json!({ "search": { "query": "rust", "fields": ["title", "bio"] } }))
        .unwrap();
    let search = params.search.unwrap();
    assert_eq!(search.query, "rust");
    assert_eq!(search.fields, vec!["title", "bio"]);
}

#[test]
fn rejects_search_with_empty_fields() {
    let err = PaginationSchema::new()
        .validate(&json!({ "search": { "query": "rust", "fields": [] } }))
        .unwrap_err();
    assert_eq!(err.issues[0].path, vec!["search", "fields"]);
}

#[test]
fn decodes_encoded_cursor_string() {
    // Build an encoded cursor via the core crate, then round-trip through validation.
    use paginator_utils::{Cursor, CursorDirection, CursorValue};
    let encoded = Cursor::new("id".into(), CursorValue::Int(42), CursorDirection::After)
        .encode()
        .unwrap();

    let params = PaginationSchema::new()
        .validate(&json!({ "cursor": encoded }))
        .unwrap();
    let cursor = params.cursor.unwrap();
    assert_eq!(cursor.field, "id");
    assert_eq!(cursor.value, CursorValue::Int(42));
}

#[test]
fn strict_rejects_unknown_keys() {
    let err = PaginationSchema::new()
        .strict()
        .validate(&json!({ "page": 1, "bogus": true }))
        .unwrap_err();
    assert!(!err.issues.is_empty());
}

#[test]
fn typescript_module_is_well_formed() {
    let ts = typescript::response_module_ts();
    assert!(ts.contains("import * as z from \"zod\";"));
    assert!(ts.contains("export const PaginationMetaSchema"));
    assert!(ts.contains("export const paginated"));
    assert!(ts.contains("data: z.array(item)"));
    assert!(ts.contains("meta: PaginationMetaSchema"));

    let v3 = typescript::response_module_ts_with(typescript::ZodVersion::V3);
    assert!(v3.contains("import { z } from \"zod\";"));
}
