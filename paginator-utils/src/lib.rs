mod cursor;
mod filter;
mod params;
mod response;
mod search;

pub use cursor::{Cursor, CursorDirection, CursorValue};
pub use filter::{Filter, FilterOperator, FilterValue};
pub use params::{PaginationParams, SortDirection};
pub use response::{PaginatorResponse, PaginatorResponseMeta};
pub use search::SearchParams;

/// Trait for types that can be converted to PaginationParams
pub trait IntoPaginationParams {
    fn into_pagination_params(self) -> PaginationParams;
}

impl IntoPaginationParams for PaginationParams {
    fn into_pagination_params(self) -> PaginationParams {
        self
    }
}

impl IntoPaginationParams for &PaginationParams {
    fn into_pagination_params(self) -> PaginationParams {
        self.clone()
    }
}
