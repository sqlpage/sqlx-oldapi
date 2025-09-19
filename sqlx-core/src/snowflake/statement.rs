use crate::column::Column;
use crate::snowflake::{Snowflake, SnowflakeArguments, SnowflakeColumn, SnowflakeTypeInfo};
use crate::statement::Statement;
use crate::HashMap;
use std::borrow::Cow;
use std::sync::Arc;

/// Implementation of [`Statement`] for Snowflake.
#[derive(Debug, Clone)]
pub struct SnowflakeStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) columns: Arc<Vec<SnowflakeColumn>>,
    pub(crate) column_names: Arc<HashMap<String, usize>>,
    pub(crate) parameters: usize,
}

impl<'q> SnowflakeStatement<'q> {
    pub(crate) fn new(sql: Cow<'q, str>, columns: Vec<SnowflakeColumn>, parameters: usize) -> Self {
        let column_names: HashMap<String, usize> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| (col.name().to_lowercase(), i))
            .collect();

        Self {
            sql,
            columns: Arc::new(columns),
            column_names: Arc::new(column_names),
            parameters,
        }
    }
}

impl<'q> Statement<'q> for SnowflakeStatement<'q> {
    type Database = Snowflake;

    fn to_owned(&self) -> SnowflakeStatement<'static> {
        SnowflakeStatement {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            columns: Arc::clone(&self.columns),
            column_names: Arc::clone(&self.column_names),
            parameters: self.parameters,
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<either::Either<&[SnowflakeTypeInfo], usize>> {
        Some(either::Either::Right(self.parameters))
    }

    fn columns(&self) -> &[SnowflakeColumn] {
        &self.columns
    }

    impl_statement_query!(SnowflakeArguments);
}
