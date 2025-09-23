use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use bigdecimal::{BigDecimal, FromPrimitive};
use odbc_api::DataType;
use std::str::FromStr;

impl Type<Odbc> for BigDecimal {
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

impl<'q> Encode<'q, Odbc> for BigDecimal {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for BigDecimal {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(int) = value.int {
            return Ok(BigDecimal::from(int));
        }
        if let Some(float) = value.float {
            return Ok(BigDecimal::from_f64(float).ok_or(format!("bad float: {}", float))?);
        }
        if let Some(text) = value.text {
            return Ok(BigDecimal::from_str(text).map_err(|e| format!("bad decimal text: {}", e))?);
        }
        if let Some(bytes) = value.blob {
            return Ok(BigDecimal::parse_bytes(bytes, 10)
                .ok_or(format!("bad base10 bytes: {:?}", bytes))?);
        }
        Err(format!("ODBC: cannot decode BigDecimal: {:?}", value).into())
    }
}
