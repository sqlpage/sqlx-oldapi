use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;

impl Type<Snowflake> for u16 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Smallint)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Smallint
                | crate::snowflake::type_info::SnowflakeType::Integer
                | crate::snowflake::type_info::SnowflakeType::Bigint
                | crate::snowflake::type_info::SnowflakeType::Number
        )
    }
}

impl Type<Snowflake> for u32 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Integer)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Integer
                | crate::snowflake::type_info::SnowflakeType::Bigint
                | crate::snowflake::type_info::SnowflakeType::Number
        )
    }
}

impl Type<Snowflake> for u64 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Bigint)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Bigint
                | crate::snowflake::type_info::SnowflakeType::Number
        )
    }
}

macro_rules! impl_uint_encode {
    ($T:ty) => {
        impl<'q> Encode<'q, Snowflake> for $T {
            fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
                buf.buffer.extend_from_slice(self.to_string().as_bytes());
                IsNull::No
            }
        }
    };
}

macro_rules! impl_uint_decode {
    ($T:ty) => {
        impl<'r> Decode<'r, Snowflake> for $T {
            fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
                match value.value {
                    Some(serde_json::Value::Number(n)) => {
                        if let Some(i) = n.as_u64() {
                            <$T>::try_from(i).map_err(|_| "number out of range".into())
                        } else if let Some(f) = n.as_f64() {
                            if f.fract() == 0.0 && f >= 0.0 {
                                <$T>::try_from(f as u64).map_err(|_| "number out of range".into())
                            } else {
                                Err("expected non-negative integer".into())
                            }
                        } else {
                            Err("invalid number".into())
                        }
                    }
                    Some(serde_json::Value::String(s)) => {
                        s.parse::<$T>().map_err(|_| "invalid integer string".into())
                    }
                    None => Err("unexpected null".into()),
                    _ => Err("expected number".into()),
                }
            }
        }
    };
}

impl_uint_encode!(u16);
impl_uint_encode!(u32);
impl_uint_encode!(u64);

impl_uint_decode!(u16);
impl_uint_decode!(u32);
impl_uint_decode!(u64);