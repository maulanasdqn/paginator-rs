//! paginator-zod: validate pagination input with zod-rs, and emit the TypeScript
//! response schema. Run with:
//!   cargo run -p paginator-zod --example validate_and_codegen

use paginator_zod::{typescript, Locale, PaginationSchema};
use serde_json::json;

fn main() {
    let schema = PaginationSchema::new()
        .max_per_page(100)
        .allowed_sort_fields(["name", "created_at"])
        .allowed_filter_fields(["status", "age"]);

    println!("=== Valid input ===");
    let input = json!({
        "page": 2,
        "per_page": 20,
        "sort_by": "created_at",
        "sort_direction": "desc",
        "filters": [
            { "field": "status", "operator": "eq", "value": "active" },
            { "field": "age", "operator": "gt", "value": 18 }
        ],
        "search": { "query": "rust", "fields": ["name"] }
    });
    match schema.validate(&input) {
        Ok(params) => println!(
            "page={} per_page={} sort={:?} {:?} filters={} search={:?}",
            params.page,
            params.per_page,
            params.sort_by,
            params.sort_direction,
            params.filters.len(),
            params.search.map(|s| s.query),
        ),
        Err(e) => println!("unexpected error:{e}"),
    }

    println!("\n=== per_page over the max ===");
    if let Err(e) = schema.validate(&json!({ "per_page": 500 })) {
        println!("{e}");
    }

    println!("\n=== sort field not in the allow-list ===");
    if let Err(e) = schema.validate(&json!({ "sort_by": "password" })) {
        println!("{e}");
    }

    println!("\n=== invalid filter operator ===");
    if let Err(e) = schema.validate(&json!({
        "filters": [{ "field": "status", "operator": "DROP", "value": 1 }]
    })) {
        println!("{e}");
    }

    println!("\n=== localized (Arabic) error ===");
    if let Err(e) = schema.validate(&json!({ "per_page": 0 })) {
        println!("{}", e.local(Locale::Ar));
    }

    println!("\n=== defaults applied when omitted ===");
    let params = schema.validate(&json!({})).unwrap();
    println!("page={} per_page={}", params.page, params.per_page);

    println!("\n=== Generated TypeScript response schema ===\n");
    println!("{}", typescript::response_module_ts());
}
