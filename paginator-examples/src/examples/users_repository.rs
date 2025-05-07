use paginator::{PaginatorResponse, PaginatorResponseMeta, PaginatorTrait};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsersData {
    pub id: u32,
    pub name: String,
    pub email: String,
}

impl UsersData {
    pub fn new(id: u32, name: String, email: String) -> Self {
        UsersData { id, name, email }
    }
}

impl PaginatorTrait<UsersData> for Vec<UsersData> {
    fn paginate(&self) -> PaginatorResponse<UsersData> {
        let mut data = self.clone();
        data.sort_by(|a, b| a.id.cmp(&b.id));
        PaginatorResponse {
            data,
            meta: PaginatorResponseMeta {
                page: 1,
                per_page: self.len() as u32,
                total: self.len() as u32,
            },
        }
    }
}
