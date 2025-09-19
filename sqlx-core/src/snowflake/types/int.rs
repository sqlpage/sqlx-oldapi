use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;

impl Type<Snowflake> for i8 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Tinyint)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Tinyint
                | crate::snowflake::type_info::SnowflakeType::Byteint
        )
    }
}

impl Type<Snowflake> for i16 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Smallint)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Smallint
                | crate::snowflake::type_info::SnowflakeType::Tinyint
                | crate::snowflake::type_info::SnowflakeType::Byteint
        )
    }
}

impl Type<Snowflake> for i32 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Integer)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Integer
                | crate::snowflake::type_info::SnowflakeType::Int
                | crate::snowflake::type_info::SnowflakeType::Smallint
                | crate::snowflake::type_info::SnowflakeType::Tinyint
                | crate::snowflake::type_info::SnowflakeType::Byteint
        )
    }
}

impl Type<Snowflake> for i64 {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Bigint)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Bigint
                | crate::snowflake::type_info::SnowflakeType::Integer
                | crate::snowflake::type_info::SnowflakeType::Int
                | crate::snowflake::type_info::SnowflakeType::Smallint
                | crate::snowflake::type_info::SnowflakeType::Tinyint
                | crate::snowflake::type_info::SnowflakeType::Byteint
                | crate::snowflake::type_info::SnowflakeType::Number
        )
    }
}

macro_rules! impl_int_encode {
    ($T:ty) => {
        impl<'q> Encode<'q, Snowflake> for $T {
            fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
                buf.buffer.extend_from_slice(self.to_string().as_bytes());
                IsNull::No
            }
        }
    };
}

macro_rules! impl_int_decode {
    ($T:ty) => {
        impl<'r> Decode<'r, Snowflake> for $T {
            fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
                match value.value {
                    Some(serde_json::Value::Number(n)) => {
                        if let Some(i) = n.as_i64() {
                            <$T>::try_from(i).map_err(|_| "number out of range".into())
                        } else if let Some(f) = n.as_f64() {
                            if f.fract() == 0.0 {
                                <$T>::try_from(f as i64).map_err(|_| "number out of range".into())
                            } else {
                                Err("expected integer, got float".into())
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

impl_int_encode!(i8);
impl_int_encode!(i16);
impl_int_encode!(i32);
impl_int_encode!(i64);

impl_int_decode!(i8);
impl_int_decode!(i16);
impl_int_decode!(i32);
impl_int_decode!(i64);