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
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for BigDecimal {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = String::decode(value)?;
        Ok(BigDecimal::from_str(&s)?)
    }
}
