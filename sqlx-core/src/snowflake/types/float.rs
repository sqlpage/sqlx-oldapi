use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;

impl Type<Snowflake> for f32 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Float)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Float
                | crate::snowflake::type_info::SnowflakeType::Float4
                | crate::snowflake::type_info::SnowflakeType::Real
        )
    }
}

impl Type<Snowflake> for f64 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Double)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Double
                | crate::snowflake::type_info::SnowflakeType::DoublePrecision
                | crate::snowflake::type_info::SnowflakeType::Float8
                | crate::snowflake::type_info::SnowflakeType::Float
                | crate::snowflake::type_info::SnowflakeType::Float4
                | crate::snowflake::type_info::SnowflakeType::Real
                | crate::snowflake::type_info::SnowflakeType::Number
        )
    }
}

impl<'q> Encode<'q, Snowflake> for f32 {
    fn encode_by_ref(
        &self,
        buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer,
    ) -> IsNull {
        buf.buffer.extend_from_slice(self.to_string().as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, Snowflake> for f64 {
    fn encode_by_ref(
        &self,
        buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer,
    ) -> IsNull {
        buf.buffer.extend_from_slice(self.to_string().as_bytes());
        IsNull::No
    }
}

impl<'r> Decode<'r, Snowflake> for f32 {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::Number(n)) => n
                .as_f64()
                .map(|f| f as f32)
                .ok_or_else(|| "number out of range for f32".into()),
            Some(serde_json::Value::String(s)) => {
                s.parse::<f32>().map_err(|_| "invalid float string".into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected number".into()),
        }
    }
}

impl<'r> Decode<'r, Snowflake> for f64 {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::Number(n)) => n
                .as_f64()
                .ok_or_else(|| "number out of range for f64".into()),
            Some(serde_json::Value::String(s)) => {
                s.parse::<f64>().map_err(|_| "invalid float string".into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected number".into()),
        }
    }
}
