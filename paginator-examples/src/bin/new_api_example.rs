use paginator_rs::{Paginator, FilterBuilder, SearchBuilder, CursorBuilder};
use paginator_utils::{FilterValue, CursorValue, PaginationParams, IntoPaginationParams};

fn main() {
    println!("=== NEW FLUENT BUILDER API ===");

    // 1. Basic usage of the new fluent API
    let params = Paginator::new()
        .page(1)
        .per_page(20)
        .sort().asc("created_at")
        .filter()
            .eq("status", FilterValue::String("active".to_string()))
            .gte("age", FilterValue::Int(18))
            .like("name", "%john%")
            .apply()
        .search()
            .query("test query")
            .fields(["name", "email"])
            .exact(false)
            .case_sensitive(false)
            .apply()
        .cursor()
            .after("id", CursorValue::Int(150))
            .apply()
        .disable_total_count()
        .build();

    println!("✅ Complete parameters created: {:?}", params);

    // 2. Standalone FilterBuilder
    let standalone_filters = FilterBuilder::new()
        .eq("status", FilterValue::String("active".to_string()))
        .gte("score", FilterValue::Int(100))
        .between(
            "price",
            FilterValue::Float(10.0),
            FilterValue::Float(100.0)
        )
        .build();

    println!("✅ Standalone filters: {:?}", standalone_filters);

    // 3. Standalone SearchBuilder
    let standalone_search = SearchBuilder::new()
        .query("john doe")
        .fields(["name", "description"])
        .exact(true)
        .build();

    println!("✅ Standalone search: {:?}", standalone_search);

    // 4. Standalone CursorBuilder
    let standalone_cursor = CursorBuilder::new()
        .after("id", CursorValue::Int(42))
        .build();

    println!("✅ Standalone cursor: {:?}", standalone_cursor);

    // 5. Comparing with old API (showing backward compatibility)
    let old_params = paginator_rs::PaginatorBuilder::new()
        .page(1)
        .per_page(10)
        .sort_by("created_at")
        .sort_desc()
        .filter_eq("status", FilterValue::String("active".to_string()))
        .build();

    println!("✅ Old API still works: {:?}", old_params);

    // 6. Multiple filter usage
    let multiple_filters = Paginator::new()
        .filter()
            .eq("type", FilterValue::String("user".to_string()))
            .apply()
        .filter()
            .gte("created_at", FilterValue::String("2023-01-01".to_string()))
            .apply()
        .build();

    println!("✅ Multiple filters: {:?}", multiple_filters);

    println!("\n=== DIRECT BUILDERS USAGE (FOR paginate_query) ===");

    // Mock function to simulate how paginate_query now accepts builders directly
    fn mock_paginate_query<P: IntoPaginationParams>(
        query: &str,
        params: P,
    ) -> PaginationParams {
        let pagination_params = params.into_pagination_params();
        println!("Query: {}", query);
        println!("Converted params: {:?}", pagination_params);
        pagination_params
    }

    // 7. Using FilterBuilder directly (no need for Paginator wrapper)
    let filter_only = FilterBuilder::new()
        .eq("status", FilterValue::String("active".to_string()))
        .gte("age", FilterValue::Int(18))
        .like("name", "%john%");

    let _result = mock_paginate_query(
        "SELECT * FROM users",
        filter_only
    );
    println!("✅ FilterBuilder used directly in paginate_query");

    // 8. Using SearchBuilder directly
    let search_only = SearchBuilder::new()
        .query("john")
        .fields(["name", "email"])
        .case_sensitive(false);

    let _result = mock_paginate_query(
        "SELECT * FROM users",
        search_only
    );
    println!("✅ SearchBuilder used directly in paginate_query");

    // 9. Using CursorBuilder directly
    let cursor_only = CursorBuilder::new()
        .after("id", CursorValue::Int(100));

    let _result = mock_paginate_query(
        "SELECT * FROM users",
        cursor_only
    );
    println!("✅ CursorBuilder used directly in paginate_query");

    // 10. Using full Paginator (fluent API) directly
    let full_paginator = Paginator::new()
        .page(2)
        .per_page(15)
        .sort().desc("created_at")
        .filter()
            .eq("active", FilterValue::Bool(true))
            .apply();

    let _result = mock_paginate_query(
        "SELECT * FROM users",
        full_paginator
    );
    println!("✅ Full Paginator used directly in paginate_query");

    // 11. Traditional PaginationParams still works
    let traditional_params = PaginationParams {
        page: 1,
        per_page: 20,
        ..Default::default()
    };

    let _result = mock_paginate_query(
        "SELECT * FROM users",
        &traditional_params
    );
    println!("✅ Traditional PaginationParams still works");

    println!("\n=== EVERYTHING WORKING! ===");
    println!("Now you can pass any builder directly to paginate_query!");
}