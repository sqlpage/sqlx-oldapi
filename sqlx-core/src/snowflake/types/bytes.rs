use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;
use base64;

impl Type<Snowflake> for [u8] {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Binary)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Binary
                | crate::snowflake::type_info::SnowflakeType::Varbinary
        )
    }
}

impl Type<Snowflake> for Vec<u8> {
    fn type_info() -> SnowflakeTypeInfo {
        <[u8] as Type<Snowflake>>::type_info()
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        <[u8] as Type<Snowflake>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Snowflake> for &'q [u8] {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        // Encode as base64 string for JSON transport
        let encoded = base64::encode(self);
        buf.buffer.extend_from_slice(encoded.as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, Snowflake> for Vec<u8> {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        <&[u8] as Encode<Snowflake>>::encode_by_ref(&self.as_slice(), buf)
    }
}

impl<'r> Decode<'r, Snowflake> for Vec<u8> {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => {
                // Snowflake returns binary data as base64-encoded strings
                base64::decode(s).map_err(|e| format!("invalid base64: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected string (base64 encoded binary)".into()),
        }
    }
}