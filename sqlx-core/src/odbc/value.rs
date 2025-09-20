use crate::decode::Decode;
use crate::error::BoxDynError;
use crate::odbc::{Odbc, OdbcTypeInfo};
use crate::value::{Value, ValueRef};
use std::borrow::Cow;

pub struct OdbcValueRef<'r> {
    pub(crate) type_info: OdbcTypeInfo,
    pub(crate) is_null: bool,
    pub(crate) text: Option<&'r str>,
    pub(crate) blob: Option<&'r [u8]>,
    pub(crate) int: Option<i64>,
    pub(crate) float: Option<f64>,
}

#[derive(Clone)]
pub struct OdbcValue {
    pub(crate) type_info: OdbcTypeInfo,
    pub(crate) is_null: bool,
    pub(crate) data: Vec<u8>,
}

impl<'r> ValueRef<'r> for OdbcValueRef<'r> {
    type Database = Odbc;

    fn to_owned(&self) -> OdbcValue {
        OdbcValue {
            type_info: self.type_info.clone(),
            is_null: self.is_null,
            data: self.blob.unwrap_or(&[]).to_vec(),
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }
    fn is_null(&self) -> bool {
        self.is_null
    }
}

impl Value for OdbcValue {
    type Database = Odbc;

    fn as_ref(&self) -> OdbcValueRef<'_> {
        OdbcValueRef {
            type_info: self.type_info.clone(),
            is_null: self.is_null,
            text: None,
            blob: Some(&self.data),
            int: None,
            float: None,
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }
    fn is_null(&self) -> bool {
        self.is_null
    }
}

impl<'r> Decode<'r, Odbc> for String {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.text {
            return Ok(text.to_owned());
        }
        if let Some(bytes) = value.blob {
            return Ok(std::str::from_utf8(bytes)?.to_owned());
        }
        Err("ODBC: cannot decode String".into())
    }
}

impl<'r> Decode<'r, Odbc> for &'r str {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.text {
            return Ok(text);
        }
        if let Some(bytes) = value.blob {
            return Ok(std::str::from_utf8(bytes)?);
        }
        Err("ODBC: cannot decode &str".into())
    }
}

impl<'r> Decode<'r, Odbc> for i64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(i) = value.int {
            return Ok(i);
        }
        if let Some(bytes) = value.blob {
            let s = std::str::from_utf8(bytes)?;
            return Ok(s.trim().parse()?);
        }
        Err("ODBC: cannot decode i64".into())
    }
}

impl<'r> Decode<'r, Odbc> for i32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(i64::decode(value)? as i32)
    }
}

impl<'r> Decode<'r, Odbc> for f64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(f) = value.float {
            return Ok(f);
        }
        if let Some(bytes) = value.blob {
            let s = std::str::from_utf8(bytes)?;
            return Ok(s.trim().parse()?);
        }
        Err("ODBC: cannot decode f64".into())
    }
}

impl<'r> Decode<'r, Odbc> for f32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(f64::decode(value)? as f32)
    }
}

impl<'r> Decode<'r, Odbc> for Vec<u8> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(bytes) = value.blob {
            return Ok(bytes.to_vec());
        }
        if let Some(text) = value.text {
            return Ok(text.as_bytes().to_vec());
        }
        Err("ODBC: cannot decode Vec<u8>".into())
    }
}

impl<'r> Decode<'r, Odbc> for i16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(i64::decode(value)? as i16)
    }
}

impl<'r> Decode<'r, Odbc> for i8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(i64::decode(value)? as i8)
    }
}

impl<'r> Decode<'r, Odbc> for bool {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(i) = value.int {
            return Ok(i != 0);
        }
        if let Some(bytes) = value.blob {
            let s = std::str::from_utf8(bytes)?;
            let s = s.trim();
            return Ok(match s {
                "0" | "false" | "FALSE" | "f" | "F" => false,
                "1" | "true" | "TRUE" | "t" | "T" => true,
                _ => s.parse()?,
            });
        }
        if let Some(text) = value.text {
            let text = text.trim();
            return Ok(match text {
                "0" | "false" | "FALSE" | "f" | "F" => false,
                "1" | "true" | "TRUE" | "t" | "T" => true,
                _ => text.parse()?,
            });
        }
        Err("ODBC: cannot decode bool".into())
    }
}

impl<'r> Decode<'r, Odbc> for u8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = i64::decode(value)?;
        Ok(u8::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = i64::decode(value)?;
        Ok(u16::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = i64::decode(value)?;
        Ok(u32::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = i64::decode(value)?;
        Ok(u64::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for &'r [u8] {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(bytes) = value.blob {
            return Ok(bytes);
        }
        if let Some(text) = value.text {
            return Ok(text.as_bytes());
        }
        Err("ODBC: cannot decode &[u8]".into())
    }
}

// Feature-gated decode implementations
#[cfg(feature = "chrono")]
mod chrono_decode {
    use super::*;
    use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};

    impl<'r> Decode<'r, Odbc> for NaiveDate {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(s.parse()?)
        }
    }

    impl<'r> Decode<'r, Odbc> for NaiveTime {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(s.parse()?)
        }
    }

    impl<'r> Decode<'r, Odbc> for NaiveDateTime {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(s.parse()?)
        }
    }

    impl<'r> Decode<'r, Odbc> for DateTime<Utc> {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(s.parse()?)
        }
    }

    impl<'r> Decode<'r, Odbc> for DateTime<FixedOffset> {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(s.parse()?)
        }
    }

    impl<'r> Decode<'r, Odbc> for DateTime<Local> {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(s.parse::<DateTime<Utc>>()?.with_timezone(&Local))
        }
    }
}

#[cfg(feature = "json")]
mod json_decode {
    use super::*;
    use serde_json::Value;

    impl<'r> Decode<'r, Odbc> for Value {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(serde_json::from_str(&s)?)
        }
    }
}

#[cfg(feature = "bigdecimal")]
mod bigdecimal_decode {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    impl<'r> Decode<'r, Odbc> for BigDecimal {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(BigDecimal::from_str(&s)?)
        }
    }
}

#[cfg(feature = "decimal")]
mod decimal_decode {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    impl<'r> Decode<'r, Odbc> for Decimal {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            let s = String::decode(value)?;
            Ok(Decimal::from_str(&s)?)
        }
    }
}

#[cfg(feature = "uuid")]
mod uuid_decode {
    use super::*;
    use std::str::FromStr;
    use uuid::Uuid;

    impl<'r> Decode<'r, Odbc> for Uuid {
        fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
            if let Some(bytes) = value.blob {
                if bytes.len() == 16 {
                    // Binary UUID format
                    return Ok(Uuid::from_bytes(bytes.try_into()?));
                }
                // Try as string
                let s = std::str::from_utf8(bytes)?;
                return Ok(Uuid::from_str(s)?);
            }
            let s = String::decode(value)?;
            Ok(Uuid::from_str(&s)?)
        }
    }
}
