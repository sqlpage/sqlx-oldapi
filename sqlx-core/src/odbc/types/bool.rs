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
        if let Some(bytes) = value.blob {
            let s = std::str::from_utf8(bytes)?;
            let s = s.trim();
            return Ok(match s {
                "0" | "false" | "FALSE" | "f" | "F" => false,
                "1" | "true" | "TRUE" | "t" | "T" => true,
                _ => s.parse()?,
            });
        }
        if let Some(text) = value.text {
            let text = text.trim();
            return Ok(match text {
                "0" | "false" | "FALSE" | "f" | "F" => false,
                "1" | "true" | "TRUE" | "t" | "T" => true,
                _ => text.parse()?,
            });
        }
        Err("ODBC: cannot decode bool".into())
    }
}
