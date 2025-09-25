use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use bigdecimal::BigDecimal;
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
        if let Some(int) = value.int::<i64>() {
            Ok(BigDecimal::from(int))
        } else if let Some(float) = value.float::<f64>() {
            Ok(BigDecimal::try_from(float)?)
        } else if let Some(text) = value.text() {
            let text = text.trim();
            Ok(BigDecimal::from_str(text).map_err(|e| format!("bad decimal text: {}", e))?)
        } else if let Some(bytes) = value.blob() {
            if let Ok(s) = std::str::from_utf8(bytes) {
                Ok(BigDecimal::parse_bytes(s.as_bytes(), 10)
                    .ok_or(format!("bad base10 bytes: {:?}", bytes))?)
            } else {
                Err(format!("bad utf8 bytes: {:?}", bytes).into())
            }
        } else {
            Err(format!("ODBC: cannot decode BigDecimal: {:?}", value).into())
        }
    }
}
