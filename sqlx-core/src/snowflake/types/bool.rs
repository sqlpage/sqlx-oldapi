use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;

impl Type<Snowflake> for bool {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Boolean)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Boolean
        )
    }
}

impl<'q> Encode<'q, Snowflake> for bool {
    fn encode_by_ref(
        &self,
        buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer,
    ) -> IsNull {
        buf.buffer
            .extend_from_slice(if *self { b"true" } else { b"false" });
        IsNull::No
    }
}

impl<'r> Decode<'r, Snowflake> for bool {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::Bool(b)) => Ok(*b),
            Some(serde_json::Value::String(s)) => match s.to_lowercase().as_str() {
                "true" | "t" | "yes" | "y" | "1" => Ok(true),
                "false" | "f" | "no" | "n" | "0" => Ok(false),
                _ => Err(format!("invalid boolean value: {}", s).into()),
            },
            Some(serde_json::Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    Ok(i != 0)
                } else {
                    Err("invalid boolean number value".into())
                }
            }
            None => Err("unexpected null".into()),
            _ => Err("expected boolean".into()),
        }
    }
}
