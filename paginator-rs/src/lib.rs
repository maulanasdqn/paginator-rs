use paginator_utils;
pub use paginator_utils::*;

use serde::Serialize;
use serde_json::{Value, to_value};

pub trait PaginatorTrait<T>
where
    T: Serialize,
{
    fn paginate(&self) -> PaginatorResponse<T> {
        PaginatorResponse {
            data: vec![],
            meta: PaginatorResponseMeta {
                page: 0,
                per_page: 0,
                total: 0,
            },
        }
    }

    fn paginate_json(&self) -> Value {
        to_value(self.paginate()).unwrap()
    }
}
