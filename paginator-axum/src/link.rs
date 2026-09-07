use paginator_rs::{PaginationParams, PaginatorResponseMeta, SortDirection};

/// Build an RFC 8288 `Link` header for a paginated response.
///
/// `first` and `last` (when the total is known) are page links. `prev` and
/// `next` use `prev_cursor`/`next_cursor` when the response carries them,
/// keeping `per_page`, `sort_by`, and `sort_direction` so the cursor is
/// interpreted the same way on the next request; otherwise they are page links.
pub fn create_link_header(
    base_url: &str,
    params: &PaginationParams,
    meta: &PaginatorResponseMeta,
) -> String {
    let mut links = Vec::new();

    links.push(link(base_url, &page_query(1, params), "first"));

    if meta.has_prev {
        if let Some(cursor) = &meta.prev_cursor {
            links.push(link(base_url, &cursor_query(cursor, params), "prev"));
        } else if params.page > 1 {
            links.push(link(base_url, &page_query(params.page - 1, params), "prev"));
        }
    }

    if meta.has_next {
        if let Some(cursor) = &meta.next_cursor {
            links.push(link(base_url, &cursor_query(cursor, params), "next"));
        } else {
            links.push(link(base_url, &page_query(params.page + 1, params), "next"));
        }
    }

    if let Some(total_pages) = meta.total_pages {
        links.push(link(base_url, &page_query(total_pages, params), "last"));
    }

    links.join(", ")
}

fn link(base_url: &str, query: &str, rel: &str) -> String {
    format!("<{}?{}>; rel=\"{}\"", base_url, query, rel)
}

fn page_query(page: u32, params: &PaginationParams) -> String {
    format!("page={}&per_page={}", page, params.per_page)
}

fn cursor_query(cursor: &str, params: &PaginationParams) -> String {
    let mut query = form_urlencoded::Serializer::new(String::new());
    query
        .append_pair("cursor", cursor)
        .append_pair("per_page", &params.per_page.to_string());
    if let Some(sort_by) = &params.sort_by {
        query.append_pair("sort_by", sort_by);
    }
    if let Some(direction) = params.sort_direction {
        query.append_pair(
            "sort_direction",
            match direction {
                SortDirection::Asc => "asc",
                SortDirection::Desc => "desc",
            },
        );
    }
    query.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_links() {
        let params = PaginationParams::new(2, 10);
        let meta = PaginatorResponseMeta::new(2, 10, 35);
        assert_eq!(
            create_link_header("/users", &params, &meta),
            "</users?page=1&per_page=10>; rel=\"first\", \
             </users?page=1&per_page=10>; rel=\"prev\", \
             </users?page=3&per_page=10>; rel=\"next\", \
             </users?page=4&per_page=10>; rel=\"last\""
        );
    }

    #[test]
    fn cursor_links_carry_sort() {
        let params = PaginationParams::new(1, 10)
            .with_sort("id")
            .with_direction(SortDirection::Desc);
        let meta = PaginatorResponseMeta::new_with_cursors(
            1,
            10,
            None,
            true,
            Some("NEXT".into()),
            Some("PREV".into()),
        );
        assert_eq!(
            create_link_header("/users", &params, &meta),
            "</users?page=1&per_page=10>; rel=\"first\", \
             </users?cursor=PREV&per_page=10&sort_by=id&sort_direction=desc>; rel=\"prev\", \
             </users?cursor=NEXT&per_page=10&sort_by=id&sort_direction=desc>; rel=\"next\""
        );
    }

    #[test]
    fn no_prev_link_on_first_page_without_cursor() {
        let params = PaginationParams::new(1, 10);
        let meta = PaginatorResponseMeta::new_with_cursors(1, 10, None, false, None, None);
        let header = create_link_header("/users", &params, &meta);
        assert!(!header.contains("rel=\"prev\""), "{header}");
        assert!(!header.contains("rel=\"next\""), "{header}");
    }
}
