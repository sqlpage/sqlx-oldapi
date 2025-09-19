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
