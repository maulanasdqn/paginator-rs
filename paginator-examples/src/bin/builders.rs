//! Every way to build `PaginationParams`: the legacy `PaginatorBuilder`,
//! the fluent `Paginator` API, and the standalone sub-builders.
//!
//! Run with: cargo run -p paginator-examples --bin builders

use paginator_rs::{
    CursorBuilder, CursorValue, FilterBuilder, FilterOperator, FilterValue, Paginator,
    PaginatorBuilder, SearchBuilder,
};

fn main() {
    println!("=== Legacy PaginatorBuilder: pages and sorting ===");
    let params = PaginatorBuilder::new()
        .page(2)
        .per_page(50)
        .sort_by("created_at")
        .sort_desc()
        .build();
    println!("{:#?}", params);

    println!("\n=== Legacy PaginatorBuilder: every filter helper ===");
    let params = PaginatorBuilder::new()
        .filter_eq("status", FilterValue::String("active".into()))
        .filter_ne("role", FilterValue::String("bot".into()))
        .filter_gt("age", FilterValue::Int(18))
        .filter_lt("age", FilterValue::Int(65))
        .filter_gte("score", FilterValue::Float(4.5))
        .filter_lte("retries", FilterValue::Int(3))
        .filter_like("name", "Jo%")
        .filter_ilike("email", "%@example.com")
        .filter_in(
            "role",
            vec![
                FilterValue::String("admin".into()),
                FilterValue::String("moderator".into()),
            ],
        )
        .filter_between("created_at", FilterValue::Int(2020), FilterValue::Int(2024))
        .filter_is_null("deleted_at")
        // Operators without a dedicated helper go through the generic filter()
        .filter("banned", FilterOperator::Eq, FilterValue::Bool(false))
        .build();
    println!("{} filters built", params.filters.len());

    println!("\n=== Legacy PaginatorBuilder: search variants ===");
    let fuzzy = PaginatorBuilder::new()
        .search("john", vec!["name".into(), "email".into()])
        .build();
    let exact = PaginatorBuilder::new()
        .search_exact("John Doe", vec!["name".into()])
        .build();
    let case_sensitive = PaginatorBuilder::new()
        .search_case_sensitive("John", vec!["name".into()])
        .build();
    println!("fuzzy:          {:?}", fuzzy.search);
    println!("exact:          {:?}", exact.search);
    println!("case_sensitive: {:?}", case_sensitive.search);

    println!("\n=== Legacy PaginatorBuilder: cursors and COUNT skipping ===");
    let params = PaginatorBuilder::new()
        .per_page(20)
        .sort_by("id")
        .cursor_after("id", CursorValue::Int(42))
        .disable_total_count()
        .build();
    println!(
        "cursor={:?} disable_total_count={}",
        params.cursor, params.disable_total_count
    );

    println!("\n=== Fluent Paginator API ===");
    let params = Paginator::new()
        .page(1)
        .per_page(10)
        .sort()
        .desc("created_at")
        .filter()
        .eq("status", FilterValue::String("active".into()))
        .gt("age", FilterValue::Int(18))
        .not_in("role", vec![FilterValue::String("bot".into())])
        .contains("bio", FilterValue::String("rust".into()))
        .is_not_null("email")
        .apply()
        .search()
        .query("developer")
        .fields(["title", "bio"])
        .apply()
        .build();
    println!("{:#?}", params);

    println!("\n=== Fluent Paginator API: cursor sub-builder ===");
    let params = Paginator::new()
        .per_page(20)
        .cursor()
        .after("id", CursorValue::Int(100))
        .apply()
        .disable_total_count()
        .build();
    println!("cursor={:?}", params.cursor);

    println!("\n=== Standalone sub-builders ===");
    let filters = FilterBuilder::new()
        .eq("status", FilterValue::String("active".into()))
        .between("age", FilterValue::Int(18), FilterValue::Int(65))
        .build();
    println!("filters: {:?}", filters);

    let search = SearchBuilder::new()
        .query("john")
        .fields(["name", "email"])
        .exact(false)
        .case_sensitive(false)
        .build();
    println!("search: {:?}", search);

    let cursor = CursorBuilder::new()
        .before("id", CursorValue::Int(42))
        .build();
    println!("cursor: {:?}", cursor);
}
