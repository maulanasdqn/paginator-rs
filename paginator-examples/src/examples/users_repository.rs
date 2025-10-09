use paginator_rs::{
    PaginationParams, PaginatorResponse, PaginatorResponseMeta, PaginatorResult, PaginatorTrait,
    SortDirection,
};
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
    fn paginate(&self, params: &PaginationParams) -> PaginatorResult<PaginatorResponse<UsersData>> {
        use paginator_rs::{FilterOperator, FilterValue};

        // Start with all data
        let mut data = self.clone();

        // Apply filters
        for filter in &params.filters {
            data.retain(|user| {
                let field_value = match filter.field.as_str() {
                    "id" => FilterValue::Int(user.id as i64),
                    "name" => FilterValue::String(user.name.clone()),
                    "email" => FilterValue::String(user.email.clone()),
                    _ => return true, // Unknown field, keep the item
                };

                match (&filter.operator, &filter.value) {
                    (FilterOperator::Eq, value) => field_value == *value,
                    (FilterOperator::Ne, value) => field_value != *value,
                    (FilterOperator::Gt, FilterValue::Int(v)) => {
                        if let FilterValue::Int(fv) = field_value {
                            fv > *v
                        } else {
                            false
                        }
                    }
                    (FilterOperator::Lt, FilterValue::Int(v)) => {
                        if let FilterValue::Int(fv) = field_value {
                            fv < *v
                        } else {
                            false
                        }
                    }
                    (FilterOperator::Gte, FilterValue::Int(v)) => {
                        if let FilterValue::Int(fv) = field_value {
                            fv >= *v
                        } else {
                            false
                        }
                    }
                    (FilterOperator::Lte, FilterValue::Int(v)) => {
                        if let FilterValue::Int(fv) = field_value {
                            fv <= *v
                        } else {
                            false
                        }
                    }
                    (FilterOperator::Like | FilterOperator::ILike, FilterValue::String(pattern)) => {
                        if let FilterValue::String(fv) = field_value {
                            let pattern_clean = pattern.replace('%', "");
                            fv.to_lowercase().contains(&pattern_clean.to_lowercase())
                        } else {
                            false
                        }
                    }
                    (FilterOperator::In, FilterValue::Array(values)) => {
                        values.contains(&field_value)
                    }
                    (FilterOperator::NotIn, FilterValue::Array(values)) => {
                        !values.contains(&field_value)
                    }
                    (FilterOperator::Between, FilterValue::Array(values)) => {
                        if values.len() == 2 {
                            if let (FilterValue::Int(min), FilterValue::Int(max)) = (&values[0], &values[1]) {
                                if let FilterValue::Int(fv) = field_value {
                                    fv >= *min && fv <= *max
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    _ => true, // Unknown operator, keep the item
                }
            });
        }

        // Apply search
        if let Some(ref search) = params.search {
            data.retain(|user| {
                let search_query = if search.case_sensitive {
                    search.query.clone()
                } else {
                    search.query.to_lowercase()
                };

                search.fields.iter().any(|field| {
                    let field_value = match field.as_str() {
                        "name" => Some(&user.name),
                        "email" => Some(&user.email),
                        _ => None,
                    };

                    if let Some(value) = field_value {
                        let check_value = if search.case_sensitive {
                            value.clone()
                        } else {
                            value.to_lowercase()
                        };

                        if search.exact_match {
                            check_value == search_query
                        } else {
                            check_value.contains(&search_query)
                        }
                    } else {
                        false
                    }
                })
            });
        }

        let total = data.len() as u32;

        // Sort data if sort parameters are provided
        if let Some(ref field) = params.sort_by {
            let direction = params
                .sort_direction
                .as_ref()
                .unwrap_or(&SortDirection::Asc);

            match field.as_str() {
                "id" => {
                    data.sort_by(|a, b| {
                        if direction == &SortDirection::Asc {
                            a.id.cmp(&b.id)
                        } else {
                            b.id.cmp(&a.id)
                        }
                    });
                }
                "name" => {
                    data.sort_by(|a, b| {
                        if direction == &SortDirection::Asc {
                            a.name.cmp(&b.name)
                        } else {
                            b.name.cmp(&a.name)
                        }
                    });
                }
                "email" => {
                    data.sort_by(|a, b| {
                        if direction == &SortDirection::Asc {
                            a.email.cmp(&b.email)
                        } else {
                            b.email.cmp(&a.email)
                        }
                    });
                }
                _ => {} // Unknown field, no sorting
            }
        }

        // Calculate pagination
        let offset = params.offset() as usize;
        let limit = params.limit() as usize;

        // Get paginated slice
        let end = (offset + limit).min(data.len());
        let paginated_data = if offset < data.len() {
            data[offset..end].to_vec()
        } else {
            vec![]
        };

        Ok(PaginatorResponse {
            data: paginated_data,
            meta: PaginatorResponseMeta::new(params.page, params.per_page, total),
        })
    }
}
