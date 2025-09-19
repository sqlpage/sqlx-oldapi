use crate::snowflake::{Snowflake, SnowflakeTypeInfo};
use crate::value::{Value, ValueRef};
use serde_json;

/// An owned value from Snowflake.
#[derive(Debug, Clone)]
pub struct SnowflakeValue {
    pub(crate) type_info: SnowflakeTypeInfo,
    pub(crate) value: Option<serde_json::Value>,
}

/// A borrowed value from Snowflake.
#[derive(Debug)]
pub struct SnowflakeValueRef<'r> {
    pub(crate) type_info: SnowflakeTypeInfo,
    pub(crate) value: Option<&'r serde_json::Value>,
}

impl SnowflakeValue {
    pub(crate) fn new(type_info: SnowflakeTypeInfo, value: Option<serde_json::Value>) -> Self {
        Self { type_info, value }
    }
}

impl<'r> SnowflakeValueRef<'r> {
    pub(crate) fn new(type_info: SnowflakeTypeInfo, value: Option<&'r serde_json::Value>) -> Self {
        Self { type_info, value }
    }
}

impl Value for SnowflakeValue {
    type Database = Snowflake;

    fn as_ref(&self) -> SnowflakeValueRef<'_> {
        SnowflakeValueRef {
            type_info: self.type_info.clone(),
            value: self.value.as_ref(),
        }
    }

    fn type_info(&self) -> std::borrow::Cow<'_, SnowflakeTypeInfo> {
        std::borrow::Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        self.value.is_none()
    }
}

impl<'r> ValueRef<'r> for SnowflakeValueRef<'r> {
    type Database = Snowflake;

    fn to_owned(&self) -> SnowflakeValue {
        SnowflakeValue {
            type_info: self.type_info.clone(),
            value: self.value.cloned(),
        }
    }

    fn type_info(&self) -> std::borrow::Cow<'_, SnowflakeTypeInfo> {
        std::borrow::Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        self.value.is_none()
    }
}
