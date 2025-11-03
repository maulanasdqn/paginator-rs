use paginator_rs::{Filter, FilterOperator, FilterValue, PaginationParams};
use sqlx::query_builder::QueryBuilder;
use sqlx::Database;

pub trait QueryBuilderExt<'args, DB: Database> {
    fn push_filter(&mut self, filter: &Filter) -> &mut Self;
    fn push_filters(&mut self, params: &PaginationParams) -> &mut Self;
    fn push_search(&mut self, params: &PaginationParams) -> &mut Self;
}

impl<'args, DB: Database> QueryBuilderExt<'args, DB> for QueryBuilder<'args, DB>
where
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    f64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    bool: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    fn push_filter(&mut self, filter: &Filter) -> &mut Self {
        self.push(&filter.field);

        match &filter.operator {
            FilterOperator::Eq => {
                self.push(" = ");
                bind_value(self, &filter.value);
            }
            FilterOperator::Ne => {
                self.push(" != ");
                bind_value(self, &filter.value);
            }
            FilterOperator::Gt => {
                self.push(" > ");
                bind_value(self, &filter.value);
            }
            FilterOperator::Lt => {
                self.push(" < ");
                bind_value(self, &filter.value);
            }
            FilterOperator::Gte => {
                self.push(" >= ");
                bind_value(self, &filter.value);
            }
            FilterOperator::Lte => {
                self.push(" <= ");
                bind_value(self, &filter.value);
            }
            FilterOperator::Like => {
                self.push(" LIKE ");
                bind_value(self, &filter.value);
            }
            FilterOperator::ILike => {
                self.push(" ILIKE ");
                bind_value(self, &filter.value);
            }
            FilterOperator::In => {
                if let FilterValue::Array(values) = &filter.value {
                    self.push(" IN (");
                    let mut separated = self.separated(", ");
                    for value in values {
                        match value {
                            FilterValue::String(s) => {
                                separated.push_bind(s.clone());
                            }
                            FilterValue::Int(i) => {
                                separated.push_bind(*i);
                            }
                            FilterValue::Float(f) => {
                                separated.push_bind(*f);
                            }
                            FilterValue::Bool(b) => {
                                separated.push_bind(*b);
                            }
                            _ => {}
                        };
                    }
                    self.push(")");
                }
            }
            FilterOperator::NotIn => {
                if let FilterValue::Array(values) = &filter.value {
                    self.push(" NOT IN (");
                    let mut separated = self.separated(", ");
                    for value in values {
                        match value {
                            FilterValue::String(s) => {
                                separated.push_bind(s.clone());
                            }
                            FilterValue::Int(i) => {
                                separated.push_bind(*i);
                            }
                            FilterValue::Float(f) => {
                                separated.push_bind(*f);
                            }
                            FilterValue::Bool(b) => {
                                separated.push_bind(*b);
                            }
                            _ => {}
                        };
                    }
                    self.push(")");
                }
            }
            FilterOperator::IsNull => {
                self.push(" IS NULL");
            }
            FilterOperator::IsNotNull => {
                self.push(" IS NOT NULL");
            }
            FilterOperator::Between => {
                if let FilterValue::Array(arr) = &filter.value {
                    if arr.len() == 2 {
                        self.push(" BETWEEN ");
                        bind_value(self, &arr[0]);
                        self.push(" AND ");
                        bind_value(self, &arr[1]);
                    }
                }
            }
            FilterOperator::Contains => {
                self.push(" @> ");
                bind_value(self, &filter.value);
            }
        }

        self
    }

    fn push_filters(&mut self, params: &PaginationParams) -> &mut Self {
        if !params.filters.is_empty() {
            for filter in &params.filters {
                self.push(" AND ");
                self.push_filter(filter);
            }
        }
        self
    }

    fn push_search(&mut self, params: &PaginationParams) -> &mut Self {
        if let Some(ref search) = params.search {
            if !search.fields.is_empty() {
                self.push(" AND (");

                for (idx, field) in search.fields.iter().enumerate() {
                    if idx > 0 {
                        self.push(" OR ");
                    }

                    let pattern = if search.exact_match {
                        search.query.clone()
                    } else {
                        format!("%{}%", search.query)
                    };

                    if search.case_sensitive {
                        self.push(field);
                        self.push(" LIKE ");
                        self.push_bind(pattern);
                    } else {
                        self.push("LOWER(");
                        self.push(field);
                        self.push(") LIKE LOWER(");
                        self.push_bind(pattern);
                        self.push(")");
                    }
                }

                self.push(")");
            }
        }
        self
    }
}

fn bind_value<'args, DB: Database>(builder: &mut QueryBuilder<'args, DB>, value: &FilterValue)
where
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    f64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    bool: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    match value {
        FilterValue::String(s) => {
            builder.push_bind(s.clone());
        }
        FilterValue::Int(i) => {
            builder.push_bind(*i);
        }
        FilterValue::Float(f) => {
            builder.push_bind(*f);
        }
        FilterValue::Bool(b) => {
            builder.push_bind(*b);
        }
        FilterValue::Null => {
            builder.push("NULL");
        }
        FilterValue::Array(_) => {}
    }
}
