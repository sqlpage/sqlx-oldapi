use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;
use uuid::Uuid;

impl Type<Snowflake> for Uuid {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Varchar)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Varchar
                | crate::snowflake::type_info::SnowflakeType::String
                | crate::snowflake::type_info::SnowflakeType::Text
        )
    }
}

impl<'q> Encode<'q, Snowflake> for Uuid {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        buf.buffer.extend_from_slice(self.to_string().as_bytes());
        IsNull::No
    }
}

impl<'r> Decode<'r, Snowflake> for Uuid {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => {
                Uuid::parse_str(s).map_err(|e| format!("invalid UUID: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected string for UUID".into()),
        }
    }
}