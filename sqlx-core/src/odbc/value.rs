use crate::odbc::{Odbc, OdbcTypeInfo};
use crate::value::{Value, ValueRef};
use std::borrow::Cow;

#[derive(Debug)]
pub struct OdbcValueRef<'r> {
    pub(crate) type_info: OdbcTypeInfo,
    pub(crate) is_null: bool,
    pub(crate) text: Option<&'r str>,
    pub(crate) blob: Option<&'r [u8]>,
    pub(crate) int: Option<i64>,
    pub(crate) float: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct OdbcValue {
    pub(crate) type_info: OdbcTypeInfo,
    pub(crate) is_null: bool,
    pub(crate) text: Option<String>,
    pub(crate) blob: Option<Vec<u8>>,
    pub(crate) int: Option<i64>,
    pub(crate) float: Option<f64>,
}

impl<'r> ValueRef<'r> for OdbcValueRef<'r> {
    type Database = Odbc;

    fn to_owned(&self) -> OdbcValue {
        OdbcValue {
            type_info: self.type_info.clone(),
            is_null: self.is_null,
            text: self.text.map(|s| s.to_string()),
            blob: self.blob.map(|b| b.to_vec()),
            int: self.int,
            float: self.float,
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
            text: self.text.as_deref(),
            blob: self.blob.as_deref(),
            int: self.int,
            float: self.float,
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }
    fn is_null(&self) -> bool {
        self.is_null
    }
}

// Decode implementations have been moved to the types module

#[cfg(feature = "any")]
impl<'r> From<OdbcValueRef<'r>> for crate::any::AnyValueRef<'r> {
    fn from(value: OdbcValueRef<'r>) -> Self {
        crate::any::AnyValueRef {
            type_info: crate::any::AnyTypeInfo::from(value.type_info.clone()),
            kind: crate::any::value::AnyValueRefKind::Odbc(value),
        }
    }
}

#[cfg(feature = "any")]
impl From<OdbcValue> for crate::any::AnyValue {
    fn from(value: OdbcValue) -> Self {
        crate::any::AnyValue {
            type_info: crate::any::AnyTypeInfo::from(value.type_info.clone()),
            kind: crate::any::value::AnyValueKind::Odbc(value),
        }
    }
}
