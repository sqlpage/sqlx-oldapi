use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo, SnowflakeValueRef};
use crate::types::Type;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};

impl Type<Snowflake> for NaiveDate {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Date)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(ty.r#type(), crate::snowflake::type_info::SnowflakeType::Date)
    }
}

impl Type<Snowflake> for NaiveTime {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Time)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(ty.r#type(), crate::snowflake::type_info::SnowflakeType::Time)
    }
}

impl Type<Snowflake> for NaiveDateTime {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::Timestamp)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::Timestamp
                | crate::snowflake::type_info::SnowflakeType::TimestampNtz
                | crate::snowflake::type_info::SnowflakeType::Datetime
        )
    }
}

impl Type<Snowflake> for DateTime<Utc> {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::TimestampTz)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::TimestampTz
                | crate::snowflake::type_info::SnowflakeType::Timestamp
        )
    }
}

impl Type<Snowflake> for DateTime<FixedOffset> {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::TimestampTz)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::TimestampTz
                | crate::snowflake::type_info::SnowflakeType::Timestamp
        )
    }
}

impl Type<Snowflake> for DateTime<Local> {
    fn type_info() -> SnowflakeTypeInfo {
        SnowflakeTypeInfo::new(crate::snowflake::type_info::SnowflakeType::TimestampLtz)
    }

    fn compatible(ty: &SnowflakeTypeInfo) -> bool {
        matches!(
            ty.r#type(),
            crate::snowflake::type_info::SnowflakeType::TimestampLtz
                | crate::snowflake::type_info::SnowflakeType::Timestamp
        )
    }
}

// Basic encode implementations for chrono types
impl<'q> Encode<'q, Snowflake> for NaiveDate {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        buf.buffer.extend_from_slice(self.format("%Y-%m-%d").to_string().as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, Snowflake> for NaiveTime {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        buf.buffer.extend_from_slice(self.format("%H:%M:%S%.f").to_string().as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, Snowflake> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        buf.buffer.extend_from_slice(self.format("%Y-%m-%d %H:%M:%S%.f").to_string().as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, Snowflake> for DateTime<Utc> {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        buf.buffer.extend_from_slice(self.format("%Y-%m-%d %H:%M:%S%.f %z").to_string().as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, Snowflake> for DateTime<FixedOffset> {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        buf.buffer.extend_from_slice(self.format("%Y-%m-%d %H:%M:%S%.f %z").to_string().as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, Snowflake> for DateTime<Local> {
    fn encode_by_ref(&self, buf: &mut crate::snowflake::arguments::SnowflakeArgumentBuffer) -> IsNull {
        buf.buffer.extend_from_slice(self.format("%Y-%m-%d %H:%M:%S%.f %z").to_string().as_bytes());
        IsNull::No
    }
}

// Basic decode implementations for chrono types
impl<'r> Decode<'r, Snowflake> for NaiveDate {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|e| format!("invalid date format: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected string for date".into()),
        }
    }
}

impl<'r> Decode<'r, Snowflake> for NaiveTime {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => {
                NaiveTime::parse_from_str(s, "%H:%M:%S%.f")
                    .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M:%S"))
                    .map_err(|e| format!("invalid time format: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected string for time".into()),
        }
    }
}

impl<'r> Decode<'r, Snowflake> for NaiveDateTime {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => {
                NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                    .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                    .map_err(|e| format!("invalid datetime format: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected string for datetime".into()),
        }
    }
}

impl<'r> Decode<'r, Snowflake> for DateTime<Utc> {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => {
                DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f %z")
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| format!("invalid datetime format: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected string for datetime".into()),
        }
    }
}

impl<'r> Decode<'r, Snowflake> for DateTime<FixedOffset> {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => {
                DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f %z")
                    .map_err(|e| format!("invalid datetime format: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected string for datetime".into()),
        }
    }
}

impl<'r> Decode<'r, Snowflake> for DateTime<Local> {
    fn decode(value: SnowflakeValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.value {
            Some(serde_json::Value::String(s)) => {
                DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f %z")
                    .map(|dt| dt.with_timezone(&Local))
                    .map_err(|e| format!("invalid datetime format: {}", e).into())
            }
            None => Err("unexpected null".into()),
            _ => Err("expected string for datetime".into()),
        }
    }
}