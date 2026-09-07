use crate::cursor::{Cursor, CursorDirection, KeysetPlan};
use crate::params::PaginationParams;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatorResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginatorResponseMeta,
}

impl<T: Serialize> PaginatorResponse<T> {
    /// Attach `next_cursor`/`prev_cursor` keyed on `field` to an offset page, so a
    /// client can switch from page numbers to keyset pagination.
    ///
    /// This is the usual way to hand out the first cursor: serve page 1 with
    /// offset pagination, call `with_cursors("id")`, and let the client follow
    /// `next_cursor` from there. Cursors already present on the response are kept.
    /// A cursor is only attached when the corresponding side has more rows
    /// (`has_next`/`has_prev`), the page is not empty, and `field` can be read
    /// from the serialized boundary row.
    pub fn with_cursors(mut self, field: &str) -> Self {
        if self.meta.has_next && self.meta.next_cursor.is_none() {
            self.meta.next_cursor = self
                .data
                .last()
                .and_then(|row| Cursor::from_row(field, row, CursorDirection::After).ok())
                .and_then(|c| c.encode().ok());
        }
        if self.meta.has_prev && self.meta.prev_cursor.is_none() {
            self.meta.prev_cursor = self
                .data
                .first()
                .and_then(|row| Cursor::from_row(field, row, CursorDirection::Before).ok())
                .and_then(|c| c.encode().ok());
        }
        self
    }
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

fn total_pages_for(total: u32, per_page: u32) -> u32 {
    (total as f32 / per_page as f32).ceil() as u32
}

impl PaginatorResponseMeta {
    pub fn new(page: u32, per_page: u32, total: u32) -> Self {
        let total_pages = total_pages_for(total, per_page);
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
        let total_pages = total.map(|t| total_pages_for(t, per_page));
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

    /// Build the metadata for a keyset page and normalize its rows.
    ///
    /// `rows` must be what the cursor query returned with `LIMIT per_page + 1`, in
    /// query order (see [`KeysetPlan::query_sort`]). The overflow row, if any, is
    /// dropped and a `Before` page is flipped back into the caller's sort order.
    ///
    /// `has_next`/`has_prev` describe the side of the page that was probed: an
    /// `After` page knows whether more rows follow and always reports rows behind
    /// it, and a `Before` page the reverse. `next_cursor`/`prev_cursor` are derived
    /// from the last and first row and are only set when that side has more rows,
    /// the page is not empty, and the cursor field can be read from the serialized
    /// row (see [`Cursor::value_from_row`]).
    pub fn from_cursor_page<T: Serialize>(
        rows: &mut Vec<T>,
        params: &PaginationParams,
        plan: &KeysetPlan<'_>,
        total: Option<u32>,
    ) -> Self {
        let per_page = params.per_page as usize;
        let has_more = rows.len() > per_page;
        rows.truncate(per_page);
        if plan.reverse_rows() {
            rows.reverse();
        }

        let (has_next, has_prev) = match plan.cursor.direction {
            CursorDirection::After => (has_more, true),
            CursorDirection::Before => (true, has_more),
        };

        let encode = |row: &T, direction: CursorDirection| {
            plan.cursor
                .at_row(row, direction)
                .ok()
                .and_then(|c| c.encode().ok())
        };
        let next_cursor = if has_next {
            rows.last()
                .and_then(|row| encode(row, CursorDirection::After))
        } else {
            None
        };
        let prev_cursor = if has_prev {
            rows.first()
                .and_then(|row| encode(row, CursorDirection::Before))
        } else {
            None
        };

        Self {
            page: params.page,
            per_page: params.per_page,
            total,
            total_pages: total.map(|t| total_pages_for(t, params.per_page)),
            has_next,
            has_prev,
            next_cursor,
            prev_cursor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor::{Cursor, CursorValue};
    use crate::params::SortDirection;

    #[derive(Serialize, Debug, PartialEq, Clone)]
    struct Row {
        id: i64,
    }

    fn rows(ids: &[i64]) -> Vec<Row> {
        ids.iter().map(|&id| Row { id }).collect()
    }

    fn ids(rows: &[Row]) -> Vec<i64> {
        rows.iter().map(|r| r.id).collect()
    }

    fn decoded(encoded: &Option<String>) -> Option<(i64, CursorDirection)> {
        encoded.as_ref().map(|e| {
            let c = Cursor::decode(e).unwrap();
            match c.value {
                CursorValue::Int(i) => (i, c.direction),
                other => panic!("unexpected value {other:?}"),
            }
        })
    }

    fn params(cursor: Cursor, per_page: u32, page: u32) -> PaginationParams {
        PaginationParams {
            page,
            per_page,
            cursor: Some(cursor),
            ..Default::default()
        }
    }

    #[test]
    fn after_page_with_more_rows() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(3), CursorDirection::After);
        let p = params(cursor, 3, 1);
        let plan = p.keyset_plan().unwrap().unwrap();
        // Query returned per_page + 1 rows in ascending order.
        let mut data = rows(&[4, 5, 6, 7]);
        let meta = PaginatorResponseMeta::from_cursor_page(&mut data, &p, &plan, Some(10));

        assert_eq!(ids(&data), vec![4, 5, 6]);
        assert!(meta.has_next);
        assert!(meta.has_prev);
        assert_eq!(
            decoded(&meta.next_cursor),
            Some((6, CursorDirection::After))
        );
        assert_eq!(
            decoded(&meta.prev_cursor),
            Some((4, CursorDirection::Before))
        );
        assert_eq!(meta.total, Some(10));
        assert_eq!(meta.total_pages, Some(4));
        assert_eq!(meta.page, 1);
    }

    #[test]
    fn after_page_at_the_end() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(7), CursorDirection::After);
        let p = params(cursor, 3, 1);
        let plan = p.keyset_plan().unwrap().unwrap();
        let mut data = rows(&[8, 9]);
        let meta = PaginatorResponseMeta::from_cursor_page(&mut data, &p, &plan, None);

        assert_eq!(ids(&data), vec![8, 9]);
        assert!(!meta.has_next);
        assert!(meta.has_prev);
        assert!(meta.next_cursor.is_none());
        assert_eq!(
            decoded(&meta.prev_cursor),
            Some((8, CursorDirection::Before))
        );
        assert!(meta.total.is_none());
        assert!(meta.total_pages.is_none());
    }

    #[test]
    fn empty_after_page_has_no_cursors() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(99), CursorDirection::After);
        let p = params(cursor, 3, 1);
        let plan = p.keyset_plan().unwrap().unwrap();
        let mut data: Vec<Row> = vec![];
        let meta = PaginatorResponseMeta::from_cursor_page(&mut data, &p, &plan, None);
        assert!(!meta.has_next);
        assert!(meta.has_prev);
        assert!(meta.next_cursor.is_none());
        assert!(meta.prev_cursor.is_none());
    }

    #[test]
    fn before_page_is_flipped_back_into_sort_order() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(7), CursorDirection::Before);
        let p = params(cursor, 3, 1);
        let plan = p.keyset_plan().unwrap().unwrap();
        assert_eq!(plan.query_sort, SortDirection::Desc);
        // Query ran `id < 7 ORDER BY id DESC LIMIT 4`.
        let mut data = rows(&[6, 5, 4, 3]);
        let meta = PaginatorResponseMeta::from_cursor_page(&mut data, &p, &plan, None);

        assert_eq!(ids(&data), vec![4, 5, 6]);
        assert!(meta.has_next);
        assert!(meta.has_prev);
        assert_eq!(
            decoded(&meta.next_cursor),
            Some((6, CursorDirection::After))
        );
        assert_eq!(
            decoded(&meta.prev_cursor),
            Some((4, CursorDirection::Before))
        );
    }

    #[test]
    fn before_page_at_the_start() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(3), CursorDirection::Before);
        let p = params(cursor, 3, 1);
        let plan = p.keyset_plan().unwrap().unwrap();
        let mut data = rows(&[2, 1]);
        let meta = PaginatorResponseMeta::from_cursor_page(&mut data, &p, &plan, None);

        assert_eq!(ids(&data), vec![1, 2]);
        assert!(meta.has_next);
        assert!(!meta.has_prev);
        assert_eq!(
            decoded(&meta.next_cursor),
            Some((2, CursorDirection::After))
        );
        assert!(meta.prev_cursor.is_none());
    }

    #[test]
    fn before_page_with_descending_sort() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(4), CursorDirection::Before);
        let mut p = params(cursor, 2, 1);
        p.sort_direction = Some(SortDirection::Desc);
        let plan = p.keyset_plan().unwrap().unwrap();
        assert_eq!(plan.query_sort, SortDirection::Asc);
        // Caller sorts DESC, so "before 4" are the larger ids; query ran ASC.
        let mut data = rows(&[5, 6, 7]);
        let meta = PaginatorResponseMeta::from_cursor_page(&mut data, &p, &plan, None);

        assert_eq!(ids(&data), vec![6, 5]);
        assert!(meta.has_prev);
        assert_eq!(
            decoded(&meta.prev_cursor),
            Some((6, CursorDirection::Before))
        );
        assert_eq!(
            decoded(&meta.next_cursor),
            Some((5, CursorDirection::After))
        );
    }

    #[test]
    fn relative_page_is_echoed() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(0), CursorDirection::After);
        let p = params(cursor, 2, 3);
        let plan = p.keyset_plan().unwrap().unwrap();
        let mut data = rows(&[5, 6, 7]);
        let meta = PaginatorResponseMeta::from_cursor_page(&mut data, &p, &plan, None);
        assert_eq!(meta.page, 3);
        assert_eq!(ids(&data), vec![5, 6]);
        assert!(meta.has_next && meta.has_prev);
    }

    #[test]
    fn missing_field_leaves_cursors_unset() {
        #[derive(Serialize)]
        struct NoId {
            name: String,
        }
        let cursor = Cursor::new("id".into(), CursorValue::Int(0), CursorDirection::After);
        let p = params(cursor, 1, 1);
        let plan = p.keyset_plan().unwrap().unwrap();
        let mut data = vec![NoId { name: "a".into() }, NoId { name: "b".into() }];
        let meta = PaginatorResponseMeta::from_cursor_page(&mut data, &p, &plan, None);
        assert_eq!(data.len(), 1);
        assert!(meta.has_next);
        assert!(meta.next_cursor.is_none());
        assert!(meta.prev_cursor.is_none());
    }

    #[test]
    fn with_cursors_bootstraps_from_an_offset_page() {
        let response = PaginatorResponse {
            data: rows(&[1, 2, 3]),
            meta: PaginatorResponseMeta::new(1, 3, 10),
        }
        .with_cursors("id");
        assert_eq!(
            decoded(&response.meta.next_cursor),
            Some((3, CursorDirection::After))
        );
        assert!(response.meta.prev_cursor.is_none(), "page 1 has no prev");

        let response = PaginatorResponse {
            data: rows(&[4, 5, 6]),
            meta: PaginatorResponseMeta::new(2, 3, 10),
        }
        .with_cursors("id");
        assert_eq!(
            decoded(&response.meta.next_cursor),
            Some((6, CursorDirection::After))
        );
        assert_eq!(
            decoded(&response.meta.prev_cursor),
            Some((4, CursorDirection::Before))
        );

        let response = PaginatorResponse {
            data: rows(&[10]),
            meta: PaginatorResponseMeta::new(4, 3, 10),
        }
        .with_cursors("id");
        assert!(response.meta.next_cursor.is_none(), "last page has no next");
        assert_eq!(
            decoded(&response.meta.prev_cursor),
            Some((10, CursorDirection::Before))
        );
    }

    #[test]
    fn with_cursors_keeps_existing_cursors() {
        let mut meta = PaginatorResponseMeta::new(2, 1, 3);
        meta.next_cursor = Some("keep".into());
        let response = PaginatorResponse {
            data: rows(&[2]),
            meta,
        }
        .with_cursors("id");
        assert_eq!(response.meta.next_cursor.as_deref(), Some("keep"));
        assert!(response.meta.prev_cursor.is_some());
    }
}
