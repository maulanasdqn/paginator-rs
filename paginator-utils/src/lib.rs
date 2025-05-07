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
    pub total: u32,
}
