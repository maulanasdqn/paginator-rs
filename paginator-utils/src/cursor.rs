use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cursor {
    pub field: String,
    pub value: CursorValue,
    pub direction: CursorDirection,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CursorDirection {
    After,
    Before,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CursorValue {
    String(String),
    Int(i64),
    Float(f64),
    /// UUID value stored as string, will be cast to UUID in SQL
    Uuid(String),
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
        Ok(BASE64.encode(json.as_bytes()))
    }

    pub fn decode(encoded: &str) -> Result<Self, String> {
        let decoded = BASE64.decode(encoded).map_err(|e| e.to_string())?;
        let json = String::from_utf8(decoded).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
