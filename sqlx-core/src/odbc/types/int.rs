use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use odbc_api::DataType;

impl Type<Odbc> for i32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::INTEGER
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Integer
                | DataType::SmallInt
                | DataType::TinyInt
                | DataType::BigInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for i64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIGINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::BigInt
                | DataType::Integer
                | DataType::SmallInt
                | DataType::TinyInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for i16 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::SMALLINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::SmallInt
                | DataType::TinyInt
                | DataType::Integer
                | DataType::BigInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for i8 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TINYINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::TinyInt
                | DataType::SmallInt
                | DataType::Integer
                | DataType::BigInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u8 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TINYINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::TinyInt
                | DataType::SmallInt
                | DataType::Integer
                | DataType::BigInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u16 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::SMALLINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::SmallInt
                | DataType::Integer
                | DataType::BigInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::INTEGER
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Integer
                | DataType::BigInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIGINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::BigInt
                | DataType::Integer
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl<'q> Encode<'q, Odbc> for i32 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i64 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i16 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i8 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u8 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u16 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u32 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u64 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        match i64::try_from(self) {
            Ok(value) => {
                buf.push(OdbcArgumentValue::Int(value));
                crate::encode::IsNull::No
            }
            Err(_) => {
                log::warn!("u64 value {} too large for ODBC, encoding as NULL", self);
                buf.push(OdbcArgumentValue::Null);
                crate::encode::IsNull::Yes
            }
        }
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        match i64::try_from(*self) {
            Ok(value) => {
                buf.push(OdbcArgumentValue::Int(value));
                crate::encode::IsNull::No
            }
            Err(_) => {
                log::warn!("u64 value {} too large for ODBC, encoding as NULL", self);
                buf.push(OdbcArgumentValue::Null);
                crate::encode::IsNull::Yes
            }
        }
    }
}

// Helper functions for numeric parsing
fn parse_numeric_as_i64(s: &str) -> Option<i64> {
    let trimmed = s.trim();
    if let Ok(parsed) = trimmed.parse::<i64>() {
        Some(parsed)
    } else if let Ok(parsed) = trimmed.parse::<f64>() {
        Some(parsed as i64)
    } else {
        None
    }
}

fn get_text_for_numeric_parsing(value: &OdbcValueRef<'_>) -> Result<Option<String>, BoxDynError> {
    if let Some(text) = value.text {
        return Ok(Some(text.trim().to_string()));
    }
    if let Some(bytes) = value.blob {
        let s = std::str::from_utf8(bytes)?;
        return Ok(Some(s.trim().to_string()));
    }
    Ok(None)
}

impl<'r> Decode<'r, Odbc> for i64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(i) = value.int {
            return Ok(i);
        }
        if let Some(f) = value.float {
            return Ok(f as i64);
        }
        if let Some(text) = get_text_for_numeric_parsing(&value)? {
            if let Some(parsed) = parse_numeric_as_i64(&text) {
                return Ok(parsed);
            }
        }
        Err("ODBC: cannot decode i64".into())
    }
}

impl<'r> Decode<'r, Odbc> for i32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(<i64 as Decode<'r, Odbc>>::decode(value)? as i32)
    }
}

impl<'r> Decode<'r, Odbc> for i16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(<i64 as Decode<'r, Odbc>>::decode(value)? as i16)
    }
}

impl<'r> Decode<'r, Odbc> for i8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(<i64 as Decode<'r, Odbc>>::decode(value)? as i8)
    }
}

impl<'r> Decode<'r, Odbc> for u8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = <i64 as Decode<'r, Odbc>>::decode(value)?;
        Ok(u8::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = <i64 as Decode<'r, Odbc>>::decode(value)?;
        Ok(u16::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = <i64 as Decode<'r, Odbc>>::decode(value)?;
        Ok(u32::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = <i64 as Decode<'r, Odbc>>::decode(value)?;
        Ok(u64::try_from(i)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odbc::{OdbcTypeInfo, OdbcValueRef};
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

    fn create_test_value_int(value: i64, data_type: DataType) -> OdbcValueRef<'static> {
        OdbcValueRef {
            type_info: OdbcTypeInfo::new(data_type),
            is_null: false,
            text: None,
            blob: None,
            int: Some(value),
            float: None,
        }
    }

    fn create_test_value_float(value: f64, data_type: DataType) -> OdbcValueRef<'static> {
        OdbcValueRef {
            type_info: OdbcTypeInfo::new(data_type),
            is_null: false,
            text: None,
            blob: None,
            int: None,
            float: Some(value),
        }
    }

    #[test]
    fn test_i32_type_compatibility() {
        // Standard integer types
        assert!(<i32 as Type<Odbc>>::compatible(&OdbcTypeInfo::INTEGER));
        assert!(<i32 as Type<Odbc>>::compatible(&OdbcTypeInfo::SMALLINT));
        assert!(<i32 as Type<Odbc>>::compatible(&OdbcTypeInfo::TINYINT));
        assert!(<i32 as Type<Odbc>>::compatible(&OdbcTypeInfo::BIGINT));

        // DECIMAL and NUMERIC types (Snowflake compatibility)
        assert!(<i32 as Type<Odbc>>::compatible(&OdbcTypeInfo::decimal(
            10, 2
        )));
        assert!(<i32 as Type<Odbc>>::compatible(&OdbcTypeInfo::numeric(
            15, 4
        )));

        // Character types
        assert!(<i32 as Type<Odbc>>::compatible(&OdbcTypeInfo::varchar(
            None
        )));

        // Should not be compatible with binary types
        assert!(!<i32 as Type<Odbc>>::compatible(&OdbcTypeInfo::varbinary(
            None
        )));
    }

    #[test]
    fn test_i64_decode_from_text() -> Result<(), BoxDynError> {
        let value = create_test_value_text(
            "42",
            DataType::Decimal {
                precision: 10,
                scale: 0,
            },
        );
        let decoded = <i64 as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, 42);

        // Test with decimal value (should truncate)
        let value = create_test_value_text(
            "42.7",
            DataType::Decimal {
                precision: 10,
                scale: 1,
            },
        );
        let decoded = <i64 as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, 42);

        // Test with whitespace
        let value = create_test_value_text(
            "  123  ",
            DataType::Decimal {
                precision: 10,
                scale: 0,
            },
        );
        let decoded = <i64 as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, 123);

        Ok(())
    }

    #[test]
    fn test_i64_decode_from_int() -> Result<(), BoxDynError> {
        let value = create_test_value_int(42, DataType::Integer);
        let decoded = <i64 as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, 42);

        Ok(())
    }

    #[test]
    fn test_i64_decode_from_float() -> Result<(), BoxDynError> {
        let value = create_test_value_float(42.7, DataType::Double);
        let decoded = <i64 as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, 42);

        Ok(())
    }

    #[test]
    fn test_i32_decode() -> Result<(), BoxDynError> {
        let value = create_test_value_text(
            "42",
            DataType::Decimal {
                precision: 10,
                scale: 0,
            },
        );
        let decoded = <i32 as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, 42);

        // Test negative
        let value = create_test_value_text(
            "-123",
            DataType::Decimal {
                precision: 10,
                scale: 0,
            },
        );
        let decoded = <i32 as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, -123);

        Ok(())
    }

    #[test]
    fn test_u32_type_compatibility() {
        // Should be compatible with DECIMAL/NUMERIC
        assert!(<u32 as Type<Odbc>>::compatible(&OdbcTypeInfo::decimal(
            10, 2
        )));
        assert!(<u32 as Type<Odbc>>::compatible(&OdbcTypeInfo::numeric(
            15, 4
        )));

        // Standard integer types
        assert!(<u32 as Type<Odbc>>::compatible(&OdbcTypeInfo::INTEGER));
        assert!(<u32 as Type<Odbc>>::compatible(&OdbcTypeInfo::BIGINT));

        // Character types
        assert!(<u32 as Type<Odbc>>::compatible(&OdbcTypeInfo::varchar(
            None
        )));
    }

    #[test]
    fn test_u64_decode() -> Result<(), BoxDynError> {
        let value = create_test_value_text(
            "42",
            DataType::Numeric {
                precision: 20,
                scale: 0,
            },
        );
        let decoded = <u64 as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, 42);

        Ok(())
    }

    #[test]
    fn test_decode_error_handling() {
        let value = OdbcValueRef {
            type_info: OdbcTypeInfo::INTEGER,
            is_null: false,
            text: None,
            blob: None,
            int: None,
            float: None,
        };

        let result = <i64 as Decode<Odbc>>::decode(value);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "ODBC: cannot decode i64");
    }

    #[test]
    fn test_encode_i32() {
        let mut buf = Vec::new();
        let result = <i32 as Encode<Odbc>>::encode(42i32, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Int(val) = &buf[0] {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected Int argument");
        }
    }

    #[test]
    fn test_encode_u64_overflow() {
        let mut buf = Vec::new();
        let large_val = u64::MAX;
        let result = <u64 as Encode<Odbc>>::encode(large_val, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::Yes));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Null = &buf[0] {
            // Expected
        } else {
            panic!("Expected Null argument for overflow");
        }
    }

    #[test]
    fn test_all_integer_types_support_decimal() {
        let decimal_type = OdbcTypeInfo::decimal(10, 2);
        let numeric_type = OdbcTypeInfo::numeric(15, 4);

        assert!(<i8 as Type<Odbc>>::compatible(&decimal_type));
        assert!(<i8 as Type<Odbc>>::compatible(&numeric_type));

        assert!(<i16 as Type<Odbc>>::compatible(&decimal_type));
        assert!(<i16 as Type<Odbc>>::compatible(&numeric_type));

        assert!(<i32 as Type<Odbc>>::compatible(&decimal_type));
        assert!(<i32 as Type<Odbc>>::compatible(&numeric_type));

        assert!(<i64 as Type<Odbc>>::compatible(&decimal_type));
        assert!(<i64 as Type<Odbc>>::compatible(&numeric_type));

        assert!(<u8 as Type<Odbc>>::compatible(&decimal_type));
        assert!(<u8 as Type<Odbc>>::compatible(&numeric_type));

        assert!(<u16 as Type<Odbc>>::compatible(&decimal_type));
        assert!(<u16 as Type<Odbc>>::compatible(&numeric_type));

        assert!(<u32 as Type<Odbc>>::compatible(&decimal_type));
        assert!(<u32 as Type<Odbc>>::compatible(&numeric_type));

        assert!(<u64 as Type<Odbc>>::compatible(&decimal_type));
        assert!(<u64 as Type<Odbc>>::compatible(&numeric_type));
    }
}
