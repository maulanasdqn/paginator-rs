use crate::error::{PaginatorError, PaginatorResult};
use paginator_utils::{PaginationParams, PaginatorResponse, PaginatorResponseMeta};
use serde::Serialize;
use serde_json::{to_value, Value};

pub trait PaginatorTrait<T>
where
    T: Serialize,
{
    fn paginate(&self, params: &PaginationParams) -> PaginatorResult<PaginatorResponse<T>> {
        if params.page < 1 {
            return Err(PaginatorError::InvalidPage(params.page));
        }
        if params.per_page < 1 || params.per_page > 100 {
            return Err(PaginatorError::InvalidPerPage(params.per_page));
        }

        Ok(PaginatorResponse {
            data: vec![],
            meta: PaginatorResponseMeta::new(0, params.per_page, 0),
        })
    }

    fn paginate_json(&self, params: &PaginationParams) -> PaginatorResult<Value> {
        let response = self.paginate(params)?;
        to_value(response).map_err(|e| PaginatorError::SerializationError(e.to_string()))
    }
}
