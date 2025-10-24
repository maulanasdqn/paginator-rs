use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchParams {
    pub query: String,
    pub fields: Vec<String>,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub exact_match: bool,
}

impl SearchParams {
    pub fn new(query: impl Into<String>, fields: Vec<String>) -> Self {
        Self {
            query: query.into(),
            fields,
            case_sensitive: false,
            exact_match: false,
        }
    }

    pub fn with_case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    pub fn with_exact_match(mut self, exact: bool) -> Self {
        self.exact_match = exact;
        self
    }

    pub fn to_sql_where(&self) -> String {
        let pattern = if self.exact_match {
            format!("'{}'", self.query.replace('\'', "''"))
        } else {
            format!("'%{}%'", self.query.replace('\'', "''"))
        };

        let operator = if self.case_sensitive { "LIKE" } else { "ILIKE" };

        let conditions: Vec<String> = self
            .fields
            .iter()
            .map(|field| {
                if self.case_sensitive || operator == "ILIKE" {
                    format!("{} {} {}", field, operator, pattern)
                } else {
                    format!("LOWER({}) LIKE LOWER({})", field, pattern)
                }
            })
            .collect();

        format!("({})", conditions.join(" OR "))
    }
}
