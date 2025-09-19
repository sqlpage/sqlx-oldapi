use crate::column::Column;
use crate::snowflake::{Snowflake, SnowflakeTypeInfo};

/// Implementation of [`Column`] for Snowflake.
#[derive(Debug, Clone)]
pub struct SnowflakeColumn {
    pub(crate) name: String,
    pub(crate) type_info: SnowflakeTypeInfo,
    pub(crate) ordinal: usize,
}

impl SnowflakeColumn {
    pub(crate) fn new(name: String, type_info: SnowflakeTypeInfo, ordinal: usize) -> Self {
        Self {
            name,
            type_info,
            ordinal,
        }
    }
}

impl crate::column::private_column::Sealed for SnowflakeColumn {}

impl Column for SnowflakeColumn {
    type Database = Snowflake;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn type_info(&self) -> &SnowflakeTypeInfo {
        &self.type_info
    }
}
