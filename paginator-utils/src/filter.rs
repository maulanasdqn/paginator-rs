use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    Like,
    ILike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
    Contains,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FilterValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<FilterValue>),
    Null,
}

impl FilterValue {
    pub fn to_sql_string(&self) -> String {
        match self {
            FilterValue::String(s) => format!("'{}'", s.replace('\'', "''")),
            FilterValue::Int(i) => i.to_string(),
            FilterValue::Float(f) => f.to_string(),
            FilterValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            FilterValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_sql_string()).collect();
                format!("({})", items.join(", "))
            }
            FilterValue::Null => "NULL".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

impl Filter {
    pub fn new(field: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
        }
    }

    pub fn to_sql_where(&self) -> String {
        match &self.operator {
            FilterOperator::Eq => format!("{} = {}", self.field, self.value.to_sql_string()),
            FilterOperator::Ne => format!("{} != {}", self.field, self.value.to_sql_string()),
            FilterOperator::Gt => format!("{} > {}", self.field, self.value.to_sql_string()),
            FilterOperator::Lt => format!("{} < {}", self.field, self.value.to_sql_string()),
            FilterOperator::Gte => format!("{} >= {}", self.field, self.value.to_sql_string()),
            FilterOperator::Lte => format!("{} <= {}", self.field, self.value.to_sql_string()),
            FilterOperator::Like => format!("{} LIKE {}", self.field, self.value.to_sql_string()),
            FilterOperator::ILike => format!("{} ILIKE {}", self.field, self.value.to_sql_string()),
            FilterOperator::In => format!("{} IN {}", self.field, self.value.to_sql_string()),
            FilterOperator::NotIn => {
                format!("{} NOT IN {}", self.field, self.value.to_sql_string())
            }
            FilterOperator::IsNull => format!("{} IS NULL", self.field),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", self.field),
            FilterOperator::Between => {
                if let FilterValue::Array(arr) = &self.value {
                    if arr.len() == 2 {
                        return format!(
                            "{} BETWEEN {} AND {}",
                            self.field,
                            arr[0].to_sql_string(),
                            arr[1].to_sql_string()
                        );
                    }
                }
                format!("{} = {}", self.field, self.value.to_sql_string())
            }
            FilterOperator::Contains => {
                format!("{} @> {}", self.field, self.value.to_sql_string())
            }
        }
    }

    pub fn to_surrealql_where(&self) -> String {
        match &self.operator {
            FilterOperator::Eq => format!("{} = {}", self.field, self.value.to_sql_string()),
            FilterOperator::Ne => format!("{} != {}", self.field, self.value.to_sql_string()),
            FilterOperator::Gt => format!("{} > {}", self.field, self.value.to_sql_string()),
            FilterOperator::Lt => format!("{} < {}", self.field, self.value.to_sql_string()),
            FilterOperator::Gte => format!("{} >= {}", self.field, self.value.to_sql_string()),
            FilterOperator::Lte => format!("{} <= {}", self.field, self.value.to_sql_string()),
            FilterOperator::Like | FilterOperator::ILike => {
                format!("{} ~ {}", self.field, self.value.to_sql_string())
            }
            FilterOperator::In => format!("{} INSIDE {}", self.field, self.value.to_sql_string()),
            FilterOperator::NotIn => {
                format!("{} NOT INSIDE {}", self.field, self.value.to_sql_string())
            }
            FilterOperator::IsNull => format!("{} IS NULL", self.field),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", self.field),
            FilterOperator::Between => {
                if let FilterValue::Array(arr) = &self.value {
                    if arr.len() == 2 {
                        return format!(
                            "{} >= {} AND {} <= {}",
                            self.field,
                            arr[0].to_sql_string(),
                            self.field,
                            arr[1].to_sql_string()
                        );
                    }
                }
                format!("{} = {}", self.field, self.value.to_sql_string())
            }
            FilterOperator::Contains => {
                format!("{} CONTAINS {}", self.field, self.value.to_sql_string())
            }
        }
    }
}
