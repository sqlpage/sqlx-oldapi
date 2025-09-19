use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;
use bigdecimal::BigDecimal;
use std::str::FromStr;

impl Type<Snowflake> for BigDecimal {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Number)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Number
                | crate::snowflake::type_info::SnowflakeType::Decimal
                | crate::snowflake::type_info::SnowflakeType::Numeric
        )
    }
}

impl<'q> Encode<'q, Snowflake> for BigDecimal {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        buf.buffer.extend_from_slice(self.to_string().as_bytes());
        IsNull::No
    }
}

impl<'r> Decode<'r, Snowflake> for BigDecimal {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::Number(n)) => {
                BigDecimal::from_str(&n.to_string())
                    .map_err(|e| format!("invalid decimal: {}", e).into())
            }
            Some(serde_json::Value::String(s)) => {
                BigDecimal::from_str(s)
                    .map_err(|e| format!("invalid decimal string: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected number or string for decimal".into()),
        }
    }
}