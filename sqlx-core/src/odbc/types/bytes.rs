use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;

impl Type<Odbc> for Vec<u8> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varbinary(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_binary_data() || ty.data_type().accepts_character_data()
        // Allow decoding from character types too
    }
}

impl<'q> Encode<'q, Odbc> for Vec<u8> {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.clone()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for &'q [u8] {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.to_vec()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.to_vec()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for Vec<u8> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(<&[u8] as Decode<'r, Odbc>>::decode(value)?.to_vec())
    }
}

impl<'r> Decode<'r, Odbc> for &'r [u8] {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(bytes) = value.blob {
            return Ok(bytes);
        }
        if let Some(text) = value.text {
            return Ok(text.as_bytes());
        }
        Err(format!("ODBC: cannot decode {:?} as &[u8]", value).into())
    }
}

impl Type<Odbc> for [u8] {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varbinary(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_binary_data() || ty.data_type().accepts_character_data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odbc::{OdbcTypeInfo, OdbcValueRef};
    use crate::type_info::TypeInfo;
    use odbc_api::DataType;

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

    fn create_test_value_blob(data: &'static [u8], data_type: DataType) -> OdbcValueRef<'static> {
        OdbcValueRef {
            type_info: OdbcTypeInfo::new(data_type),
            is_null: false,
            text: None,
            blob: Some(data),
            int: None,
            float: None,
        }
    }

    #[test]
    fn test_vec_u8_type_compatibility() {
        // Should be compatible with binary types
        assert!(<Vec<u8> as Type<Odbc>>::compatible(
            &OdbcTypeInfo::varbinary(None)
        ));
        assert!(<Vec<u8> as Type<Odbc>>::compatible(&OdbcTypeInfo::binary(
            None
        )));

        // Should be compatible with character types (for hex decoding)
        assert!(<Vec<u8> as Type<Odbc>>::compatible(&OdbcTypeInfo::varchar(
            None
        )));
        assert!(<Vec<u8> as Type<Odbc>>::compatible(&OdbcTypeInfo::char(
            None
        )));

        // Should not be compatible with numeric types
        assert!(!<Vec<u8> as Type<Odbc>>::compatible(&OdbcTypeInfo::INTEGER));
    }

    #[test]
    fn test_vec_u8_decode_from_blob() -> Result<(), BoxDynError> {
        let test_data = b"Hello, ODBC!";
        let value = create_test_value_blob(test_data, DataType::Varbinary { length: None });
        let decoded = <Vec<u8> as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, test_data.to_vec());

        Ok(())
    }

    #[test]
    fn test_vec_u8_decode_from_raw_text() -> Result<(), BoxDynError> {
        let text = "Hello, World!";
        let value = create_test_value_text(text, DataType::Varchar { length: None });
        let decoded = <Vec<u8> as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, text.as_bytes().to_vec());

        Ok(())
    }

    #[test]
    fn test_slice_u8_decode_from_blob() -> Result<(), BoxDynError> {
        let test_data = b"Hello, ODBC!";
        let value = create_test_value_blob(test_data, DataType::Varbinary { length: None });
        let decoded = <&[u8] as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, test_data);

        Ok(())
    }

    #[test]
    fn test_slice_u8_decode_from_text() -> Result<(), BoxDynError> {
        let text = "Hello";
        let value = create_test_value_text(text, DataType::Varchar { length: None });
        let decoded = <&[u8] as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, text.as_bytes());

        Ok(())
    }

    #[test]
    fn test_vec_u8_encode() {
        let mut buf = Vec::new();
        let data = vec![65, 66, 67, 68, 69]; // "ABCDE"
        let result = <Vec<u8> as Encode<Odbc>>::encode(data, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Bytes(bytes) = &buf[0] {
            assert_eq!(*bytes, vec![65, 66, 67, 68, 69]);
        } else {
            panic!("Expected Bytes argument");
        }
    }

    #[test]
    fn test_slice_u8_encode() {
        let mut buf = Vec::new();
        let data: &[u8] = &[72, 101, 108, 108, 111]; // "Hello"
        let result = <&[u8] as Encode<Odbc>>::encode(data, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Bytes(bytes) = &buf[0] {
            assert_eq!(*bytes, vec![72, 101, 108, 108, 111]);
        } else {
            panic!("Expected Bytes argument");
        }
    }

    #[test]
    fn test_decode_error_handling() {
        let value = OdbcValueRef {
            type_info: OdbcTypeInfo::varbinary(None),
            is_null: false,
            text: None,
            blob: None,
            int: None,
            float: None,
        };
        assert!(<Vec<u8> as Decode<'_, Odbc>>::decode(value).is_err());
    }

    #[test]
    fn test_type_info() {
        let type_info = <Vec<u8> as Type<Odbc>>::type_info();
        assert_eq!(type_info.name(), "VARBINARY");
        assert!(matches!(
            type_info.data_type(),
            DataType::Varbinary { length: None }
        ));

        let type_info = <[u8] as Type<Odbc>>::type_info();
        assert_eq!(type_info.name(), "VARBINARY");
        assert!(matches!(
            type_info.data_type(),
            DataType::Varbinary { length: None }
        ));
    }
}
