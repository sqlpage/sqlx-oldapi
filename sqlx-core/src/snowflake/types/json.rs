use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Json;
use crate::types::Type;

impl<T> Type<Snowflake> for Json<T> {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Variant)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Variant
                | crate::snowflake::type_info::SnowflakeType::Object
                | crate::snowflake::type_info::SnowflakeType::Array
        )
    }
}


impl<'q, T> Encode<'q, Snowflake> for Json<T>
where
    T: serde::Serialize,
{
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        let json_string = serde_json::to_string(&self.0)
            .unwrap_or_else(|_| "null".to_string());
        buf.buffer.extend_from_slice(json_string.as_bytes());
        IsNull::No
    }
}


impl<'r, T> Decode<'r, Snowflake> for Json<T>
where
    T: 'r + serde::de::DeserializeOwned,
{
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(json_val) => {
                serde_json::from_value(json_val.clone())
                    .map(Json)
                    .map_err(|e| format!("invalid JSON: {}", e).into())
            }
            None => Err("unexpected null".into()),
        }
    }
}
