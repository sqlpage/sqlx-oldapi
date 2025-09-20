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
