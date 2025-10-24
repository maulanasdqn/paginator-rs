use paginator_rs::{Filter, FilterOperator, FilterValue};

pub fn parse_filter(filter_str: &str) -> Option<Filter> {
    let parts: Vec<&str> = filter_str.splitn(3, ':').collect();
    if parts.len() < 3 {
        return None;
    }

    let field = parts[0].to_string();
    let operator = match parts[1] {
        "eq" => FilterOperator::Eq,
        "ne" => FilterOperator::Ne,
        "gt" => FilterOperator::Gt,
        "lt" => FilterOperator::Lt,
        "gte" => FilterOperator::Gte,
        "lte" => FilterOperator::Lte,
        "like" => FilterOperator::Like,
        "ilike" => FilterOperator::ILike,
        "in" => FilterOperator::In,
        "not_in" => FilterOperator::NotIn,
        "is_null" => FilterOperator::IsNull,
        "is_not_null" => FilterOperator::IsNotNull,
        "between" => FilterOperator::Between,
        "contains" => FilterOperator::Contains,
        _ => return None,
    };

    let value_str = parts[2];

    let value = match operator {
        FilterOperator::IsNull | FilterOperator::IsNotNull => FilterValue::Null,
        FilterOperator::In | FilterOperator::NotIn => {
            let values: Vec<FilterValue> = value_str
                .split(',')
                .filter_map(|v| {
                    let trimmed = v.trim();

                    if let Ok(i) = trimmed.parse::<i64>() {
                        Some(FilterValue::Int(i))
                    } else if let Ok(f) = trimmed.parse::<f64>() {
                        Some(FilterValue::Float(f))
                    } else if trimmed == "true" || trimmed == "false" {
                        Some(FilterValue::Bool(trimmed == "true"))
                    } else {
                        Some(FilterValue::String(trimmed.to_string()))
                    }
                })
                .collect();
            FilterValue::Array(values)
        }
        FilterOperator::Between => {
            let values: Vec<FilterValue> = value_str
                .split(',')
                .filter_map(|v| {
                    let trimmed = v.trim();
                    if let Ok(i) = trimmed.parse::<i64>() {
                        Some(FilterValue::Int(i))
                    } else if let Ok(f) = trimmed.parse::<f64>() {
                        Some(FilterValue::Float(f))
                    } else {
                        Some(FilterValue::String(trimmed.to_string()))
                    }
                })
                .collect();
            FilterValue::Array(values)
        }
        _ => {
            if let Ok(i) = value_str.parse::<i64>() {
                FilterValue::Int(i)
            } else if let Ok(f) = value_str.parse::<f64>() {
                FilterValue::Float(f)
            } else if value_str == "true" || value_str == "false" {
                FilterValue::Bool(value_str == "true")
            } else {
                FilterValue::String(value_str.to_string())
            }
        }
    };

    Some(Filter {
        field,
        operator,
        value,
    })
}
