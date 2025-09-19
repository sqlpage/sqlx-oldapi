use crate::column::ColumnIndex;
use crate::error::Error;
use crate::row::Row;
use crate::snowflake::{Snowflake, SnowflakeColumn, SnowflakeValue, SnowflakeValueRef};
use crate::value::Value;
use std::sync::Arc;

/// Implementation of [`Row`] for Snowflake.
#[derive(Debug)]
pub struct SnowflakeRow {
    pub(crate) values: Vec<SnowflakeValue>,
    pub(crate) columns: Arc<Vec<SnowflakeColumn>>,
}

impl SnowflakeRow {
    pub(crate) fn new(values: Vec<SnowflakeValue>, columns: Arc<Vec<SnowflakeColumn>>) -> Self {
        Self { values, columns }
    }
}

impl crate::row::private_row::Sealed for SnowflakeRow {}

impl Row for SnowflakeRow {
    type Database = Snowflake;

    fn columns(&self) -> &[SnowflakeColumn] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<SnowflakeValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        Ok(self.values[index].as_ref())
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
