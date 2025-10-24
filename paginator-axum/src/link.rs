use paginator_rs::{PaginationParams, PaginatorResponseMeta};

pub fn create_link_header(
    base_url: &str,
    params: &PaginationParams,
    meta: &PaginatorResponseMeta,
) -> String {
    let mut links = Vec::new();

    links.push(format!(
        "<{}?page=1&per_page={}>; rel=\"first\"",
        base_url, params.per_page
    ));

    if meta.has_prev {
        links.push(format!(
            "<{}?page={}&per_page={}>; rel=\"prev\"",
            base_url,
            params.page - 1,
            params.per_page
        ));
    }

    if meta.has_next {
        links.push(format!(
            "<{}?page={}&per_page={}>; rel=\"next\"",
            base_url,
            params.page + 1,
            params.per_page
        ));
    }

    if let Some(total_pages) = meta.total_pages {
        links.push(format!(
            "<{}?page={}&per_page={}>; rel=\"last\"",
            base_url, total_pages, params.per_page
        ));
    }

    links.join(", ")
}
