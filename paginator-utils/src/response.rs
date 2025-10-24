use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatorResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginatorResponseMeta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatorResponseMeta {
    pub page: u32,
    pub per_page: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<u32>,
    pub has_next: bool,
    pub has_prev: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_cursor: Option<String>,
}

impl PaginatorResponseMeta {
    pub fn new(page: u32, per_page: u32, total: u32) -> Self {
        let total_pages = (total as f32 / per_page as f32).ceil() as u32;
        Self {
            page,
            per_page,
            total: Some(total),
            total_pages: Some(total_pages),
            has_next: page < total_pages,
            has_prev: page > 1,
            next_cursor: None,
            prev_cursor: None,
        }
    }

    pub fn new_without_total(page: u32, per_page: u32, has_next: bool) -> Self {
        Self {
            page,
            per_page,
            total: None,
            total_pages: None,
            has_next,
            has_prev: page > 1,
            next_cursor: None,
            prev_cursor: None,
        }
    }

    pub fn new_with_cursors(
        page: u32,
        per_page: u32,
        total: Option<u32>,
        has_next: bool,
        next_cursor: Option<String>,
        prev_cursor: Option<String>,
    ) -> Self {
        let total_pages = total.map(|t| (t as f32 / per_page as f32).ceil() as u32);
        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next,
            has_prev: page > 1 || prev_cursor.is_some(),
            next_cursor,
            prev_cursor,
        }
    }
}
