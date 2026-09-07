use crate::params::{PaginationParams, SortDirection};
use base64::alphabet;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use base64::engine::DecodePaddingMode;
use base64::Engine;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Cursors are emitted with the URL-safe alphabet and no padding so they can be
/// placed in a query string without percent-encoding.
const ENCODER: GeneralPurpose = GeneralPurpose::new(
    &alphabet::URL_SAFE,
    GeneralPurposeConfig::new().with_encode_padding(false),
);

/// Decoding accepts both alphabets, padded or not, so cursors issued by older
/// versions (standard alphabet, padded) keep working.
const DECODE_CONFIG: GeneralPurposeConfig =
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);
const URL_SAFE_DECODER: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, DECODE_CONFIG);
const STANDARD_DECODER: GeneralPurpose = GeneralPurpose::new(&alphabet::STANDARD, DECODE_CONFIG);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cursor {
    pub field: String,
    pub value: CursorValue,
    pub direction: CursorDirection,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CursorDirection {
    After,
    Before,
}

/// The value a cursor points at.
///
/// Serialized as a bare JSON number or string, except [`CursorValue::Uuid`],
/// which is serialized as `{"uuid": "..."}` so that the variant survives an
/// encode/decode round trip and PostgreSQL still receives the `::uuid` cast.
/// Cursors issued by older versions (a bare string for UUIDs) decode as
/// [`CursorValue::String`].
#[derive(Clone, Debug, PartialEq)]
pub enum CursorValue {
    String(String),
    Int(i64),
    Float(f64),
    /// UUID value stored as string, will be cast to UUID in SQL
    Uuid(String),
}

impl Serialize for CursorValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            CursorValue::String(s) => serializer.serialize_str(s),
            CursorValue::Int(i) => serializer.serialize_i64(*i),
            CursorValue::Float(f) => serializer.serialize_f64(*f),
            CursorValue::Uuid(u) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("uuid", u)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for CursorValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Int(i64),
            Float(f64),
            String(String),
            Uuid { uuid: String },
        }

        Ok(match Raw::deserialize(deserializer)? {
            Raw::Int(i) => CursorValue::Int(i),
            Raw::Float(f) => CursorValue::Float(f),
            Raw::String(s) => CursorValue::String(s),
            Raw::Uuid { uuid } => CursorValue::Uuid(uuid),
        })
    }
}

impl CursorValue {
    /// Convert a JSON value, typically one field of a serialized row, into a
    /// cursor value.
    ///
    /// `template` is an existing cursor value on the same field; when given, string
    /// values keep its `String`/`Uuid` variant and integers keep a `Float` variant.
    /// Without a template, strings shaped like a UUID become [`CursorValue::Uuid`].
    /// Booleans, nulls, arrays, and objects cannot be cursor values.
    pub fn from_json(value: &Value, template: Option<&CursorValue>) -> Option<Self> {
        match value {
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if matches!(template, Some(CursorValue::Float(_))) {
                        Some(CursorValue::Float(i as f64))
                    } else {
                        Some(CursorValue::Int(i))
                    }
                } else {
                    n.as_f64().map(CursorValue::Float)
                }
            }
            Value::String(s) => match template {
                Some(CursorValue::Uuid(_)) => Some(CursorValue::Uuid(s.clone())),
                Some(CursorValue::String(_)) => Some(CursorValue::String(s.clone())),
                _ if looks_like_uuid(s) => Some(CursorValue::Uuid(s.clone())),
                _ => Some(CursorValue::String(s.clone())),
            },
            _ => None,
        }
    }
}

fn looks_like_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    s.bytes().enumerate().all(|(i, b)| match i {
        8 | 13 | 18 | 23 => b == b'-',
        _ => b.is_ascii_hexdigit(),
    })
}

impl Cursor {
    pub fn new(field: String, value: CursorValue, direction: CursorDirection) -> Self {
        Self {
            field,
            value,
            direction,
        }
    }

    pub fn encode(&self) -> Result<String, String> {
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        Ok(ENCODER.encode(json.as_bytes()))
    }

    pub fn decode(encoded: &str) -> Result<Self, String> {
        let encoded = encoded.trim();
        let decoded = URL_SAFE_DECODER
            .decode(encoded)
            .or_else(|_| STANDARD_DECODER.decode(encoded))
            .map_err(|e| e.to_string())?;
        let json = String::from_utf8(decoded).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    /// Read the cursor field out of a serialized row.
    ///
    /// The row is serialized with serde_json and `field` is looked up on the
    /// resulting object. A qualified name such as `users.id` falls back to its last
    /// segment (`id`), so cursors on table-qualified columns still resolve. See
    /// [`CursorValue::from_json`] for how `template` shapes the value.
    pub fn value_from_row<T: Serialize>(
        field: &str,
        row: &T,
        template: Option<&CursorValue>,
    ) -> Result<CursorValue, String> {
        let json = serde_json::to_value(row).map_err(|e| e.to_string())?;
        let obj = json
            .as_object()
            .ok_or_else(|| "row did not serialize to a JSON object".to_string())?;
        let raw = obj
            .get(field)
            .or_else(|| field.rsplit('.').next().and_then(|short| obj.get(short)))
            .ok_or_else(|| format!("cursor field '{}' not found in row", field))?;
        CursorValue::from_json(raw, template)
            .ok_or_else(|| format!("cursor field '{}' is not a number or string", field))
    }

    /// Build a cursor anchored at `row`, for example to hand out a `next_cursor`
    /// from the last row of an offset page:
    ///
    /// ```
    /// use paginator_utils::{Cursor, CursorDirection, CursorValue};
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct User { id: i64, name: String }
    ///
    /// let last = User { id: 42, name: "Ada".into() };
    /// let next = Cursor::from_row("id", &last, CursorDirection::After).unwrap();
    /// assert_eq!(next.value, CursorValue::Int(42));
    /// ```
    pub fn from_row<T: Serialize>(
        field: impl Into<String>,
        row: &T,
        direction: CursorDirection,
    ) -> Result<Self, String> {
        let field = field.into();
        let value = Self::value_from_row(&field, row, None)?;
        Ok(Self::new(field, value, direction))
    }

    /// A cursor on the same field as `self`, anchored at `row`. The value keeps
    /// the variant of `self.value`, so a `Uuid` cursor stays a `Uuid` cursor.
    pub fn at_row<T: Serialize>(
        &self,
        row: &T,
        direction: CursorDirection,
    ) -> Result<Self, String> {
        let value = Self::value_from_row(&self.field, row, Some(&self.value))?;
        Ok(Self::new(self.field.clone(), value, direction))
    }
}

/// How a request carrying a cursor is executed.
///
/// Keyset pagination selects the rows on the far side of the cursor and takes the
/// nearest `per_page` of them. For an `After` cursor that is the natural sort
/// order. For a `Before` cursor the query runs in the *reversed* order so that
/// `LIMIT` picks the rows closest to the cursor, and the page is flipped back into
/// the caller's order afterwards (see [`PaginatorResponseMeta::from_cursor_page`]).
///
/// [`PaginatorResponseMeta::from_cursor_page`]: crate::PaginatorResponseMeta::from_cursor_page
#[derive(Clone, Debug, PartialEq)]
pub struct KeysetPlan<'a> {
    pub cursor: &'a Cursor,
    /// The order the caller wants rows in: `sort_direction`, ascending by default.
    pub sort: SortDirection,
    /// The order to put in the query. Equal to `sort` for `After` cursors and
    /// reversed for `Before` cursors.
    pub query_sort: SortDirection,
}

impl<'a> KeysetPlan<'a> {
    pub fn field(&self) -> &'a str {
        &self.cursor.field
    }

    pub fn value(&self) -> &'a CursorValue {
        &self.cursor.value
    }

    /// The comparison that selects rows on the far side of the cursor in query
    /// order: `>` when the query sorts ascending, `<` when descending.
    pub fn operator(&self) -> &'static str {
        match self.query_sort {
            SortDirection::Asc => ">",
            SortDirection::Desc => "<",
        }
    }

    /// Whether the fetched rows must be reversed to restore the caller's order.
    pub fn reverse_rows(&self) -> bool {
        self.cursor.direction == CursorDirection::Before
    }
}

impl PaginationParams {
    /// Resolve how to execute this request's cursor, if it has one.
    ///
    /// Errors when `sort_by` names a different field than the cursor: keyset
    /// pagination is only correct when rows are ordered by the cursor field.
    pub fn keyset_plan(&self) -> Result<Option<KeysetPlan<'_>>, String> {
        let Some(cursor) = &self.cursor else {
            return Ok(None);
        };
        if let Some(sort_by) = &self.sort_by {
            if sort_by != &cursor.field {
                return Err(format!(
                    "cursor field '{}' does not match sort_by '{}': keyset pagination must sort by the cursor field",
                    cursor.field, sort_by
                ));
            }
        }
        let sort = self.sort_direction.unwrap_or(SortDirection::Asc);
        let query_sort = match cursor.direction {
            CursorDirection::After => sort,
            CursorDirection::Before => sort.reversed(),
        };
        Ok(Some(KeysetPlan {
            cursor,
            sort,
            query_sort,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cursor_encode_decode_string() {
        let cursor = Cursor::new(
            "id".to_string(),
            CursorValue::String("abc123".to_string()),
            CursorDirection::After,
        );
        let encoded = cursor.encode().unwrap();
        let decoded = Cursor::decode(&encoded).unwrap();
        assert_eq!(cursor, decoded);
    }

    #[test]
    fn test_cursor_encode_decode_int() {
        let cursor = Cursor::new(
            "id".to_string(),
            CursorValue::Int(12345),
            CursorDirection::Before,
        );
        let encoded = cursor.encode().unwrap();
        let decoded = Cursor::decode(&encoded).unwrap();
        assert_eq!(cursor, decoded);
    }

    #[test]
    fn test_cursor_encode_decode_float() {
        let cursor = Cursor::new(
            "timestamp".to_string(),
            CursorValue::Float(1234567890.123),
            CursorDirection::After,
        );
        let encoded = cursor.encode().unwrap();
        let decoded = Cursor::decode(&encoded).unwrap();
        assert_eq!(cursor, decoded);
    }

    #[test]
    fn uuid_variant_survives_a_round_trip() {
        let cursor = Cursor::new(
            "id".to_string(),
            CursorValue::Uuid("550e8400-e29b-41d4-a716-446655440000".to_string()),
            CursorDirection::After,
        );
        let json = serde_json::to_string(&cursor).unwrap();
        assert!(json.contains(r#""value":{"uuid":"550e8400"#), "{json}");
        let decoded = Cursor::decode(&cursor.encode().unwrap()).unwrap();
        assert_eq!(decoded, cursor);

        // Older cursors carried the UUID as a bare string; they still decode.
        let legacy: Cursor =
            serde_json::from_str(r#"{"field":"id","value":"abc","direction":"after"}"#).unwrap();
        assert_eq!(legacy.value, CursorValue::String("abc".into()));
        let float: CursorValue = serde_json::from_str("2.5").unwrap();
        assert_eq!(float, CursorValue::Float(2.5));
        let int: CursorValue = serde_json::from_str("7").unwrap();
        assert_eq!(int, CursorValue::Int(7));
    }

    #[test]
    fn encoded_cursor_is_url_safe() {
        // A payload whose standard base64 form contains '+', '/' and '=' padding.
        let cursor = Cursor::new(
            "created_at".to_string(),
            CursorValue::String("2024-01-01T00:00:00Z??>>".to_string()),
            CursorDirection::After,
        );
        let encoded = cursor.encode().unwrap();
        assert!(
            encoded
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "not url-safe: {encoded}"
        );
        assert_eq!(Cursor::decode(&encoded).unwrap(), cursor);
    }

    #[test]
    fn decodes_legacy_standard_base64_cursors() {
        let cursor = Cursor::new(
            "id".to_string(),
            CursorValue::Int(7),
            CursorDirection::After,
        );
        let json = serde_json::to_string(&cursor).unwrap();
        let legacy = base64::engine::general_purpose::STANDARD.encode(json.as_bytes());
        assert!(
            legacy.ends_with('='),
            "test needs a padded payload: {legacy}"
        );
        assert_eq!(Cursor::decode(&legacy).unwrap(), cursor);
        assert_eq!(Cursor::decode(&format!(" {legacy}\n")).unwrap(), cursor);
    }

    #[test]
    fn rejects_garbage() {
        assert!(Cursor::decode("not-a-valid-cursor").is_err());
        assert!(Cursor::decode("").is_err());
    }

    #[test]
    fn value_from_json_infers_types() {
        assert_eq!(
            CursorValue::from_json(&json!(5), None),
            Some(CursorValue::Int(5))
        );
        assert_eq!(
            CursorValue::from_json(&json!(5), Some(&CursorValue::Float(0.0))),
            Some(CursorValue::Float(5.0))
        );
        assert_eq!(
            CursorValue::from_json(&json!(1.5), None),
            Some(CursorValue::Float(1.5))
        );
        assert_eq!(
            CursorValue::from_json(&json!("abc"), None),
            Some(CursorValue::String("abc".into()))
        );
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(
            CursorValue::from_json(&json!(uuid), None),
            Some(CursorValue::Uuid(uuid.into()))
        );
        assert_eq!(
            CursorValue::from_json(&json!(uuid), Some(&CursorValue::String(String::new()))),
            Some(CursorValue::String(uuid.into()))
        );
        assert_eq!(
            CursorValue::from_json(&json!("x"), Some(&CursorValue::Uuid(String::new()))),
            Some(CursorValue::Uuid("x".into()))
        );
        assert_eq!(CursorValue::from_json(&json!(true), None), None);
        assert_eq!(CursorValue::from_json(&json!(null), None), None);
        assert_eq!(CursorValue::from_json(&json!([1]), None), None);
    }

    #[derive(Serialize)]
    struct Row {
        id: i64,
        name: String,
    }

    #[test]
    fn from_row_reads_plain_and_qualified_fields() {
        let row = Row {
            id: 42,
            name: "Ada".into(),
        };
        let c = Cursor::from_row("id", &row, CursorDirection::After).unwrap();
        assert_eq!(c.field, "id");
        assert_eq!(c.value, CursorValue::Int(42));
        assert_eq!(c.direction, CursorDirection::After);

        let c = Cursor::from_row("users.name", &row, CursorDirection::Before).unwrap();
        assert_eq!(c.field, "users.name");
        assert_eq!(c.value, CursorValue::String("Ada".into()));

        let err = Cursor::from_row("missing", &row, CursorDirection::After).unwrap_err();
        assert!(err.contains("missing"), "{err}");
        assert!(Cursor::from_row("id", &42, CursorDirection::After).is_err());
    }

    #[test]
    fn at_row_keeps_value_variant() {
        #[derive(Serialize)]
        struct R {
            id: String,
        }
        let row = R { id: "abc".into() };
        let seed = Cursor::new(
            "id".into(),
            CursorValue::Uuid("seed".into()),
            CursorDirection::After,
        );
        let next = seed.at_row(&row, CursorDirection::Before).unwrap();
        assert_eq!(next.value, CursorValue::Uuid("abc".into()));
        assert_eq!(next.direction, CursorDirection::Before);
    }

    fn params_with(cursor: Cursor, sort_direction: Option<SortDirection>) -> PaginationParams {
        PaginationParams {
            cursor: Some(cursor),
            sort_direction,
            ..Default::default()
        }
    }

    #[test]
    fn keyset_plan_resolves_operator_and_order() {
        let after = Cursor::new("id".into(), CursorValue::Int(5), CursorDirection::After);
        let before = Cursor::new("id".into(), CursorValue::Int(5), CursorDirection::Before);

        let p = params_with(after.clone(), None);
        let plan = p.keyset_plan().unwrap().unwrap();
        assert_eq!(plan.query_sort, SortDirection::Asc);
        assert_eq!(plan.operator(), ">");
        assert!(!plan.reverse_rows());

        let p = params_with(after, Some(SortDirection::Desc));
        let plan = p.keyset_plan().unwrap().unwrap();
        assert_eq!(plan.query_sort, SortDirection::Desc);
        assert_eq!(plan.operator(), "<");

        let p = params_with(before.clone(), Some(SortDirection::Asc));
        let plan = p.keyset_plan().unwrap().unwrap();
        assert_eq!(plan.sort, SortDirection::Asc);
        assert_eq!(plan.query_sort, SortDirection::Desc);
        assert_eq!(plan.operator(), "<");
        assert!(plan.reverse_rows());

        let p = params_with(before, Some(SortDirection::Desc));
        let plan = p.keyset_plan().unwrap().unwrap();
        assert_eq!(plan.query_sort, SortDirection::Asc);
        assert_eq!(plan.operator(), ">");
    }

    #[test]
    fn keyset_plan_requires_matching_sort_field() {
        let cursor = Cursor::new("id".into(), CursorValue::Int(5), CursorDirection::After);
        let mut params = params_with(cursor, None);
        params.sort_by = Some("id".into());
        assert!(params.keyset_plan().unwrap().is_some());

        params.sort_by = Some("name".into());
        let err = params.keyset_plan().unwrap_err();
        assert!(err.contains("does not match"), "{err}");

        assert!(PaginationParams::default().keyset_plan().unwrap().is_none());
    }
}
