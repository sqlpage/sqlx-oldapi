use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use odbc_api::DataType;
use rust_decimal::Decimal;
use std::str::FromStr;

impl Type<Odbc> for Decimal {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::numeric(28, 4) // Standard precision/scale
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Numeric { .. }
                | DataType::Decimal { .. }
                | DataType::Double
                | DataType::Float { .. }
        ) || ty.data_type().accepts_character_data()
    }
}

impl<'q> Encode<'q, Odbc> for Decimal {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

// Helper function for getting text from value for decimal parsing
fn get_text_for_decimal_parsing(value: &OdbcValueRef<'_>) -> Result<Option<String>, BoxDynError> {
    if let Some(text) = value.text() {
        return Ok(Some(text.trim().to_string()));
    }
    if let Some(bytes) = value.blob() {
        if let Ok(text) = std::str::from_utf8(bytes) {
            return Ok(Some(text.trim().to_string()));
        }
    }
    Ok(None)
}

impl<'r> Decode<'r, Odbc> for Decimal {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Try integer conversion first (most precise)
        if let Some(int_val) = value.int::<i64>() {
            return Ok(Decimal::from(int_val));
        }

        // Try direct float conversion for better precision
        if let Some(float_val) = value.float::<f64>() {
            if let Ok(decimal) = Decimal::try_from(float_val) {
                return Ok(decimal);
            }
        }

        // Fall back to string parsing
        if let Some(text) = get_text_for_decimal_parsing(&value)? {
            return Ok(Decimal::from_str(&text)?);
        }

        Err("ODBC: cannot decode Decimal".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odbc::{ColumnData, OdbcTypeInfo, OdbcValueRef, OdbcValueVec};
    use crate::type_info::TypeInfo;
    use odbc_api::DataType;
    use std::str::FromStr;

    fn make_ref(value_vec: OdbcValueVec, data_type: DataType) -> OdbcValueRef<'static> {
        let column = ColumnData {
            values: value_vec,
            type_info: OdbcTypeInfo::new(data_type),
        };
        let ptr = Box::leak(Box::new(column));
        OdbcValueRef::new(ptr, 0)
    }

    fn create_test_value_text(text: &'static str, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::Text(vec![Some(text.to_string())]), data_type)
    }

    fn create_test_value_bytes(bytes: &'static [u8], data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::Binary(vec![Some(bytes.to_vec())]), data_type)
    }

    fn create_test_value_int(value: i64, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::NullableBigInt(vec![Some(value)]), data_type)
    }

    fn create_test_value_float(value: f64, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::NullableDouble(vec![Some(value)]), data_type)
    }

    #[test]
    fn test_decimal_type_compatibility() {
        // Should be compatible with decimal/numeric types
        assert!(<Decimal as Type<Odbc>>::compatible(&OdbcTypeInfo::decimal(
            10, 2
        )));
        assert!(<Decimal as Type<Odbc>>::compatible(&OdbcTypeInfo::numeric(
            15, 4
        )));

        // Should be compatible with floating point types
        assert!(<Decimal as Type<Odbc>>::compatible(&OdbcTypeInfo::DOUBLE));
        assert!(<Decimal as Type<Odbc>>::compatible(&OdbcTypeInfo::float(
            24
        )));

        // Should be compatible with character types
        assert!(<Decimal as Type<Odbc>>::compatible(&OdbcTypeInfo::varchar(
            None
        )));

        // Should not be compatible with binary types
        assert!(!<Decimal as Type<Odbc>>::compatible(
            &OdbcTypeInfo::varbinary(None)
        ));
    }

    #[test]
    fn test_decimal_decode_from_text() -> Result<(), BoxDynError> {
        let value = create_test_value_text(
            "123.456",
            DataType::Decimal {
                precision: 10,
                scale: 3,
            },
        );
        let decoded = <Decimal as Decode<Odbc>>::decode(value)?;
        let expected = Decimal::from_str("123.456")?;
        assert_eq!(decoded, expected);

        // Test with whitespace
        let value = create_test_value_text(
            "  987.654  ",
            DataType::Decimal {
                precision: 10,
                scale: 3,
            },
        );
        let decoded = <Decimal as Decode<Odbc>>::decode(value)?;
        let expected = Decimal::from_str("987.654")?;
        assert_eq!(decoded, expected);

        Ok(())
    }

    #[test]
    fn test_decimal_decode_from_int() -> Result<(), BoxDynError> {
        let value = create_test_value_int(
            42,
            DataType::Decimal {
                precision: 10,
                scale: 0,
            },
        );
        let decoded = <Decimal as Decode<Odbc>>::decode(value)?;
        let expected = Decimal::from(42);
        assert_eq!(decoded, expected);

        Ok(())
    }

    #[test]
    fn test_decimal_decode_from_float() -> Result<(), BoxDynError> {
        let value = create_test_value_float(
            123.456,
            DataType::Decimal {
                precision: 10,
                scale: 3,
            },
        );
        let decoded = <Decimal as Decode<Odbc>>::decode(value)?;

        // Check that it's approximately correct (floating point precision issues)
        let expected_str = "123.456";
        let expected = Decimal::from_str(expected_str)?;
        let diff = (decoded - expected).abs();
        assert!(diff < Decimal::from_str("0.001")?);

        Ok(())
    }

    #[test]
    fn test_decimal_decode_negative() -> Result<(), BoxDynError> {
        let value = create_test_value_text(
            "-123.456",
            DataType::Decimal {
                precision: 10,
                scale: 3,
            },
        );
        let decoded = <Decimal as Decode<Odbc>>::decode(value)?;
        let expected = Decimal::from_str("-123.456")?;
        assert_eq!(decoded, expected);

        Ok(())
    }

    #[test]
    fn test_decimal_encode() {
        let mut buf = Vec::new();
        let decimal = Decimal::from_str("123.456").unwrap();
        let result = <Decimal as Encode<Odbc>>::encode(decimal, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Text(text) = &buf[0] {
            assert_eq!(text, "123.456");
        } else {
            panic!("Expected Text argument");
        }
    }

    #[test]
    fn test_decimal_encode_by_ref() {
        let mut buf = Vec::new();
        let decimal = Decimal::from_str("987.654").unwrap();
        let result = <Decimal as Encode<Odbc>>::encode_by_ref(&decimal, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Text(text) = &buf[0] {
            assert_eq!(text, "987.654");
        } else {
            panic!("Expected Text argument");
        }
    }

    #[test]
    fn test_decimal_type_info() {
        let type_info = <Decimal as Type<Odbc>>::type_info();
        assert_eq!(type_info.name(), "NUMERIC");
        if let DataType::Numeric { precision, scale } = type_info.data_type() {
            assert_eq!(precision, 28);
            assert_eq!(scale, 4);
        } else {
            panic!("Expected Numeric data type");
        }
    }

    #[test]
    fn test_decimal_decode_error_handling() {
        let column = ColumnData {
            values: OdbcValueVec::Text(vec![Some("not_a_number".to_string())]),
            type_info: OdbcTypeInfo::decimal(10, 2),
        };
        let ptr = Box::leak(Box::new(column));
        let value = OdbcValueRef::new(ptr, 0);

        let result = <Decimal as Decode<Odbc>>::decode(value);
        assert!(result.is_err());
    }
}
