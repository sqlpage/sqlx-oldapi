use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use serde_json::Value;

impl Type<Odbc> for Value {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varchar(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_character_data()
            || ty.data_type().accepts_numeric_data()
            || ty.data_type().accepts_binary_data()
            || matches!(
                ty.data_type(),
                odbc_api::DataType::Other { .. } | odbc_api::DataType::Unknown
            )
    }
}

impl<'q> Encode<'q, Odbc> for Value {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for Value {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(bytes) = value.blob() {
            return serde_json::from_slice(bytes)
                .map_err(|e| format!("ODBC: cannot decode JSON from {:?}: {}", value, e).into());
        } else if let Some(text) = value.text() {
            return serde_json::from_str(text)
                .map_err(|e| format!("ODBC: cannot decode JSON from {:?}: {}", value, e).into());
        } else if let Some(i) = value.int::<i64>() {
            return Ok(Value::from(i));
        } else if let Some(f) = value.float::<f64>() {
            return Ok(Value::from(f));
        }

        Err(format!("ODBC: cannot decode JSON from {:?}", value).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odbc::{OdbcTypeInfo, OdbcValueRef};
    use crate::type_info::TypeInfo;
    use odbc_api::DataType;
    use serde_json::{json, Value};

    fn create_test_value_text(text: &'static str, data_type: DataType) -> OdbcValueRef<'static> {
        OdbcValueRef {
            type_info: OdbcTypeInfo::new(data_type),
            is_null: false,
            text: Some(text),
            blob: None,
            int: None,
            float: None,
        }
    }

    #[test]
    fn test_json_type_compatibility() {
        // Should be compatible with character types
        assert!(<Value as Type<Odbc>>::compatible(&OdbcTypeInfo::varchar(
            None
        )));
        assert!(<Value as Type<Odbc>>::compatible(&OdbcTypeInfo::char(None)));
    }

    #[test]
    fn test_json_decode_simple() -> Result<(), BoxDynError> {
        let json_str = r#"{"name": "test"}"#;
        let value = create_test_value_text(json_str, DataType::Varchar { length: None });
        let decoded = <Value as Decode<Odbc>>::decode(value)?;
        assert!(decoded.is_object());
        assert_eq!(decoded["name"], "test");

        Ok(())
    }

    #[test]
    fn test_json_decode_invalid() {
        let invalid_json = r#"{"invalid": json,}"#;
        let value = create_test_value_text(invalid_json, DataType::Varchar { length: None });
        let result = <Value as Decode<Odbc>>::decode(value);
        assert!(result.is_err(), "{:?} should be an error", result);
    }

    #[test]
    fn test_json_encode() {
        let mut buf = Vec::new();
        let json_val = json!({"name": "test"});
        let result = <Value as Encode<Odbc>>::encode(json_val, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Text(text) = &buf[0] {
            // Parse the encoded text back to verify it's valid JSON
            let reparsed: Value = serde_json::from_str(text).unwrap();
            assert!(reparsed.is_object());
        } else {
            panic!("Expected Text argument");
        }
    }

    #[test]
    fn test_json_type_info() {
        let type_info = <Value as Type<Odbc>>::type_info();
        assert_eq!(type_info.name(), "VARCHAR");
        assert!(matches!(
            type_info.data_type(),
            DataType::Varchar { length: None }
        ));
    }
}
