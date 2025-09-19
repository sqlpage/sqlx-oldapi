use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;

impl Type<Snowflake> for str {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Varchar)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Varchar
                | crate::snowflake::type_info::SnowflakeType::Char
                | crate::snowflake::type_info::SnowflakeType::Character
                | crate::snowflake::type_info::SnowflakeType::String
                | crate::snowflake::type_info::SnowflakeType::Text
        )
    }
}

impl Type<Snowflake> for String {
    fn type_info() -> SnowflakeTypeInfo {
        <str as Type<Snowflake>>::type_info()
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        <str as Type<Snowflake>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Snowflake> for &'q str {
    fn encode_by_ref(
        &self,
        buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer,
    ) -> IsNull {
        buf.buffer.extend_from_slice(self.as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, Snowflake> for String {
    fn encode_by_ref(
        &self,
        buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer,
    ) -> IsNull {
        buf.buffer.extend_from_slice(self.as_bytes());
        IsNull::No
    }
}

impl<'r> Decode<'r, Snowflake> for &'r str {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => Ok(s),
            Some(val) => Err(format!("expected string, got {}", val).into()),
            None => Err("unexpected null".into()),
        }
    }
}

impl<'r> Decode<'r, Snowflake> for String {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => Ok(s.clone()),
            Some(val) => Ok(val.to_string()),
            None => Err("unexpected null".into()),
        }
    }
}
