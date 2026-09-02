use paginator_utils::{Cursor, Filter, PaginationParams, SearchParams, SortDirection};
use serde_json::Value;
use zod_rs::prelude::*;
use zod_rs_util::{ValidateResult, ValidationError, ValidationResult};

const OPERATORS: [&str; 14] = [
    "eq",
    "ne",
    "gt",
    "lt",
    "gte",
    "lte",
    "like",
    "ilike",
    "in",
    "notin",
    "isnull",
    "isnotnull",
    "between",
    "contains",
];

/// A configurable schema that validates raw pagination query input against a
/// [`zod-rs`](https://crates.io/crates/zod-rs) schema and produces a validated
/// [`PaginationParams`].
///
/// Validation is type-aware and path-aware: a bad `per_page`, an unknown sort
/// field, or an invalid filter operator produces a [`ValidationResult`] with the
/// same rich, localizable error messages zod-rs gives elsewhere.
///
/// The expected input is the structured JSON form of `PaginationParams`:
///
/// ```json
/// {
///   "page": 1,
///   "per_page": 20,
///   "sort_by": "created_at",
///   "sort_direction": "desc",
///   "filters": [{ "field": "status", "operator": "eq", "value": "active" }],
///   "search": { "query": "rust", "fields": ["title", "bio"] },
///   "disable_total_count": false,
///   "cursor": "<base64>"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PaginationSchema {
    default_page: u32,
    default_per_page: u32,
    max_per_page: u32,
    allowed_sort_fields: Option<Vec<String>>,
    allowed_filter_fields: Option<Vec<String>>,
    strict: bool,
}

impl Default for PaginationSchema {
    fn default() -> Self {
        Self {
            default_page: 1,
            default_per_page: 20,
            max_per_page: 100,
            allowed_sort_fields: None,
            allowed_filter_fields: None,
            strict: false,
        }
    }
}

impl PaginationSchema {
    pub fn new() -> Self {
        Self::default()
    }

    /// Page number used when the input omits `page` (default 1).
    pub fn default_page(mut self, page: u32) -> Self {
        self.default_page = page.max(1);
        self
    }

    /// Page size used when the input omits `per_page` (default 20).
    pub fn default_per_page(mut self, per_page: u32) -> Self {
        self.default_per_page = per_page;
        self
    }

    /// Maximum accepted `per_page`. Larger values fail validation (default 100).
    pub fn max_per_page(mut self, max: u32) -> Self {
        self.max_per_page = max.max(1);
        self
    }

    /// Restrict `sort_by` to this set of fields. Any other value fails validation.
    pub fn allowed_sort_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_sort_fields = Some(fields.into_iter().map(Into::into).collect());
        self
    }

    /// Restrict filter `field` values to this set. Any other value fails validation.
    pub fn allowed_filter_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_filter_fields = Some(fields.into_iter().map(Into::into).collect());
        self
    }

    /// Reject unknown top-level keys instead of ignoring them.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// The underlying zod-rs schema for the pagination envelope. Exposed so it can
    /// be composed into a larger request schema or reused directly.
    pub fn schema(&self) -> ObjectSchema {
        let mut s = object()
            .optional_field("page", number().int().min(1.0))
            .optional_field(
                "per_page",
                number().int().min(1.0).max(self.max_per_page as f64),
            )
            .optional_field(
                "sort_direction",
                union::<String>()
                    .variant(literal("asc"))
                    .variant(literal("desc")),
            )
            .optional_field("disable_total_count", boolean())
            .optional_field(
                "search",
                object()
                    .field("query", string())
                    .field("fields", array(string()).min(1)),
            )
            .optional_field("filters", array(self.filter_schema()));

        s = match &self.allowed_sort_fields {
            Some(fields) => s.optional_field("sort_by", literal_union(fields)),
            None => s.optional_field("sort_by", string()),
        };

        if self.strict {
            s = s.strict();
        }
        s
    }

    fn filter_schema(&self) -> ObjectSchema {
        let field_schema = match &self.allowed_filter_fields {
            Some(fields) => literal_union(fields),
            None => union::<String>().variant(string()),
        };

        let mut operators = union::<String>();
        for op in OPERATORS {
            operators = operators.variant(literal(op));
        }

        object()
            .field("field", field_schema)
            .field("operator", operators)
    }

    /// Validate raw JSON and produce a normalized [`PaginationParams`].
    ///
    /// Returns a [`ValidationResult`] (zod-rs errors, with paths) on failure.
    pub fn validate(&self, value: &Value) -> ValidateResult<PaginationParams> {
        // Type, enum, bound, and allow-list checks with path-aware errors.
        self.schema().validate(value)?;

        let obj = value.as_object().ok_or_else(|| {
            ValidationResult::with_error(ValidationError::invalid_type(
                zod_rs_util::ValidationType::Object,
                value.into(),
            ))
        })?;

        let page = obj
            .get("page")
            .and_then(Value::as_u64)
            .map(|p| p as u32)
            .unwrap_or(self.default_page)
            .max(1);

        let per_page = obj
            .get("per_page")
            .and_then(Value::as_u64)
            .map(|p| p as u32)
            .unwrap_or(self.default_per_page)
            .clamp(1, self.max_per_page);

        let sort_by = obj
            .get("sort_by")
            .and_then(Value::as_str)
            .map(str::to_string);

        let sort_direction = match obj.get("sort_direction") {
            Some(v) => Some(from_field::<SortDirection>("sort_direction", v)?),
            None => None,
        };

        let filters: Vec<Filter> = match obj.get("filters") {
            Some(v) => from_field::<Vec<Filter>>("filters", v)?,
            None => Vec::new(),
        };

        let search = match obj.get("search") {
            Some(v) => Some(from_field::<SearchParams>("search", v)?),
            None => None,
        };

        let disable_total_count = obj
            .get("disable_total_count")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let cursor =
            match obj.get("cursor") {
                Some(Value::String(encoded)) => Some(Cursor::decode(encoded).map_err(|e| {
                    ValidationResult::from_issue("cursor", ValidationError::custom(e))
                })?),
                Some(v) if !v.is_null() => Some(from_field::<Cursor>("cursor", v)?),
                _ => None,
            };

        Ok(PaginationParams {
            page,
            per_page,
            sort_by,
            sort_direction,
            filters,
            search,
            disable_total_count,
            cursor,
        })
    }
}

/// A union of string literals, for restricting a field to a fixed set of values.
fn literal_union(values: &[String]) -> UnionSchema<String> {
    let mut u = union::<String>();
    for v in values {
        u = u.variant(literal(v.clone()));
    }
    u
}

/// Deserialize an already-validated sub-value, mapping serde failures to a
/// path-scoped validation error rather than panicking.
fn from_field<T: serde::de::DeserializeOwned>(field: &str, value: &Value) -> ValidateResult<T> {
    serde_json::from_value::<T>(value.clone())
        .map_err(|e| ValidationResult::from_issue(field, ValidationError::custom(e.to_string())))
}

trait FromIssue {
    fn from_issue(field: &str, error: ValidationError) -> Self;
}

impl FromIssue for ValidationResult {
    fn from_issue(field: &str, error: ValidationError) -> Self {
        let mut result = ValidationResult::new();
        result.add_error_at_path(vec![field.to_string()], error);
        result
    }
}
