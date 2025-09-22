use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use odbc_api::DataType;

impl Type<Odbc> for bool {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Bit
                | DataType::TinyInt
                | DataType::SmallInt
                | DataType::Integer
                | DataType::BigInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
                | DataType::Real
                | DataType::Float { .. }
                | DataType::Double
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl<'q> Encode<'q, Odbc> for bool {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(if self { 1 } else { 0 }));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(if *self { 1 } else { 0 }));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for bool {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(i) = value.int {
            return Ok(i != 0);
        }

        // Handle float values (from DECIMAL/NUMERIC types)
        if let Some(f) = value.float {
            return Ok(f != 0.0);
        }

        if let Some(text) = value.text {
            let text = text.trim();
            // Try exact string matches first
            return Ok(match text {
                "0" | "0.0" | "false" | "FALSE" | "f" | "F" => false,
                "1" | "1.0" | "true" | "TRUE" | "t" | "T" => true,
                _ => {
                    // Try parsing as number first
                    if let Ok(num) = text.parse::<f64>() {
                        num != 0.0
                    } else if let Ok(num) = text.parse::<i64>() {
                        num != 0
                    } else {
                        // Fall back to string parsing
                        text.parse()?
                    }
                }
            });
        }

        if let Some(bytes) = value.blob {
            let s = std::str::from_utf8(bytes)?;
            let s = s.trim();
            return Ok(match s {
                "0" | "0.0" | "false" | "FALSE" | "f" | "F" => false,
                "1" | "1.0" | "true" | "TRUE" | "t" | "T" => true,
                _ => {
                    // Try parsing as number first
                    if let Ok(num) = s.parse::<f64>() {
                        num != 0.0
                    } else if let Ok(num) = s.parse::<i64>() {
                        num != 0
                    } else {
                        // Fall back to string parsing
                        s.parse()?
                    }
                }
            });
        }

        Err("ODBC: cannot decode bool".into())
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
    fn test_bool_type_compatibility() {
        // Standard boolean types
        assert!(<bool as Type<Odbc>>::compatible(&OdbcTypeInfo::BIT));
        assert!(<bool as Type<Odbc>>::compatible(&OdbcTypeInfo::TINYINT));

        // DECIMAL and NUMERIC types (Snowflake compatibility)
        assert!(<bool as Type<Odbc>>::compatible(&OdbcTypeInfo::decimal(
            1, 0
        )));
        assert!(<bool as Type<Odbc>>::compatible(&OdbcTypeInfo::numeric(
            1, 0
        )));

        // Floating point types
        assert!(<bool as Type<Odbc>>::compatible(&OdbcTypeInfo::DOUBLE));
        assert!(<bool as Type<Odbc>>::compatible(&OdbcTypeInfo::REAL));

        // Character types
        assert!(<bool as Type<Odbc>>::compatible(&OdbcTypeInfo::varchar(
            None
        )));

        // Should not be compatible with binary types
        assert!(!<bool as Type<Odbc>>::compatible(&OdbcTypeInfo::varbinary(
            None
        )));
    }

    #[test]
    fn test_bool_decode_from_decimal_text() -> Result<(), BoxDynError> {
        let value = create_test_value_text(
            "1",
            DataType::Decimal {
                precision: 1,
                scale: 0,
            },
        );
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, true);

        let value = create_test_value_text(
            "0",
            DataType::Decimal {
                precision: 1,
                scale: 0,
            },
        );
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, false);

        // Test with decimal values
        let value = create_test_value_text(
            "1.0",
            DataType::Decimal {
                precision: 2,
                scale: 1,
            },
        );
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, true);

        let value = create_test_value_text(
            "0.0",
            DataType::Decimal {
                precision: 2,
                scale: 1,
            },
        );
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, false);

        Ok(())
    }

    #[test]
    fn test_bool_decode_from_float() -> Result<(), BoxDynError> {
        let value = create_test_value_float(1.0, DataType::Double);
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, true);

        let value = create_test_value_float(0.0, DataType::Double);
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, false);

        let value = create_test_value_float(42.5, DataType::Double);
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, true);

        Ok(())
    }

    #[test]
    fn test_bool_decode_from_int() -> Result<(), BoxDynError> {
        let value = create_test_value_int(1, DataType::Integer);
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, true);

        let value = create_test_value_int(0, DataType::Integer);
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, false);

        let value = create_test_value_int(-1, DataType::Integer);
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, true);

        Ok(())
    }

    #[test]
    fn test_bool_decode_string_variants() -> Result<(), BoxDynError> {
        // Test various string representations
        let test_cases = vec![
            ("true", true),
            ("TRUE", true),
            ("t", true),
            ("T", true),
            ("false", false),
            ("FALSE", false),
            ("f", false),
            ("F", false),
        ];

        for (input, expected) in test_cases {
            let value = create_test_value_text(input, DataType::Varchar { length: None });
            let decoded = <bool as Decode<Odbc>>::decode(value)?;
            assert_eq!(decoded, expected, "Failed for input: {}", input);
        }

        Ok(())
    }

    #[test]
    fn test_bool_decode_with_whitespace() -> Result<(), BoxDynError> {
        let value = create_test_value_text(
            "  1  ",
            DataType::Decimal {
                precision: 1,
                scale: 0,
            },
        );
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, true);

        let value = create_test_value_text(
            "  0  ",
            DataType::Decimal {
                precision: 1,
                scale: 0,
            },
        );
        let decoded = <bool as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, false);

        Ok(())
    }

    #[test]
    fn test_bool_encode() {
        let mut buf = Vec::new();
        let result = <bool as Encode<Odbc>>::encode(true, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Int(val) = &buf[0] {
            assert_eq!(*val, 1);
        } else {
            panic!("Expected Int argument");
        }

        let mut buf = Vec::new();
        let result = <bool as Encode<Odbc>>::encode(false, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Int(val) = &buf[0] {
            assert_eq!(*val, 0);
        } else {
            panic!("Expected Int argument");
        }
    }

    #[test]
    fn test_bool_type_info() {
        let type_info = <bool as Type<Odbc>>::type_info();
        assert_eq!(type_info.name(), "BIT");
        assert!(matches!(type_info.data_type(), DataType::Bit));
    }

    #[test]
    fn test_bool_decode_error_handling() {
        let value = OdbcValueRef {
            type_info: OdbcTypeInfo::BIT,
            is_null: false,
            text: None,
            blob: None,
            int: None,
            float: None,
        };

        let result = <bool as Decode<Odbc>>::decode(value);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "ODBC: cannot decode bool");
    }
}
