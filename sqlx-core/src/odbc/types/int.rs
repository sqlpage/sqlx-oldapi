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
    if let Some(text) = value.text() {
        return Ok(Some(text.trim().to_string()));
    }
    if let Some(bytes) = value.blob() {
        let s = std::str::from_utf8(bytes)?;
        return Ok(Some(s.trim().to_string()));
    }
    Ok(None)
}

impl<'r> Decode<'r, Odbc> for i64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.try_int::<i64>()?)
    }
}

impl<'r> Decode<'r, Odbc> for i32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.try_int::<i32>()?)
    }
}

impl<'r> Decode<'r, Odbc> for i16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.try_int::<i16>()?)
    }
}

impl<'r> Decode<'r, Odbc> for i8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.try_int::<i8>()?)
    }
}

impl<'r> Decode<'r, Odbc> for u8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.try_int::<u8>()?)
    }
}

impl<'r> Decode<'r, Odbc> for u16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.try_int::<u16>()?)
    }
}

impl<'r> Decode<'r, Odbc> for u32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.try_int::<u32>()?)
    }
}

impl<'r> Decode<'r, Odbc> for u64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.try_int::<u64>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odbc::{
        ColumnData, OdbcBatch, OdbcColumn, OdbcTypeInfo, OdbcValueRef, OdbcValueVec,
    };
    use odbc_api::DataType;
    use std::sync::Arc;

    fn make_ref(value_vec: OdbcValueVec, data_type: DataType) -> OdbcValueRef<'static> {
        let column = ColumnData {
            values: value_vec,
            type_info: OdbcTypeInfo::new(data_type),
            nulls: vec![false],
        };
        let column_data = vec![Arc::new(column)];
        let batch = OdbcBatch {
            columns: Arc::new([OdbcColumn {
                name: "test".to_string(),
                type_info: OdbcTypeInfo::new(data_type),
                ordinal: 0,
            }]),
            column_data,
        };
        let batch_ptr = Box::leak(Box::new(batch));
        OdbcValueRef::new(batch_ptr, 0, 0)
    }

    fn create_test_value_text(text: &'static str, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::Text(vec![text.to_string()]), data_type)
    }

    fn create_test_value_blob(data: &'static [u8], data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::Binary(vec![data.to_vec()]), data_type)
    }

    fn create_test_value_int(value: i64, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::BigInt(vec![value]), data_type)
    }

    fn create_test_value_float(value: f64, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::Double(vec![value]), data_type)
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
        let decoded = <i64 as Decode<Odbc>>::decode(value).expect("Failed to decode 42");
        assert_eq!(decoded, 42);

        // Test with whitespace
        let value = create_test_value_text(
            "  123  ",
            DataType::Decimal {
                precision: 10,
                scale: 0,
            },
        );
        let decoded = <i64 as Decode<Odbc>>::decode(value).expect("Failed to decode '  123  '");
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
        let result = <i64 as Decode<Odbc>>::decode(value);
        // i64 should not be compatible with DOUBLE type
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatched types"));

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
        let column = ColumnData {
            values: OdbcValueVec::Text(vec!["not_a_number".to_string()]),
            type_info: OdbcTypeInfo::INTEGER,
            nulls: vec![false],
        };
        let column_data = vec![Arc::new(column)];
        let batch = OdbcBatch {
            columns: Arc::new([OdbcColumn {
                name: "test".to_string(),
                type_info: OdbcTypeInfo::INTEGER,
                ordinal: 0,
            }]),
            column_data,
        };
        let batch_ptr = Box::leak(Box::new(batch));
        let value = OdbcValueRef::new(batch_ptr, 0, 0);

        let result = <i64 as Decode<Odbc>>::decode(value);
        assert!(result.is_err());
        // The new implementation gives more specific error messages
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("mismatched types") || error_msg.contains("ODBC: cannot decode")
        );
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
