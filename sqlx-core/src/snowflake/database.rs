use crate::database::{Database, HasArguments, HasStatement, HasStatementCache, HasValueRef};
use crate::snowflake::arguments::SnowflakeArgumentBuffer;
use crate::snowflake::value::{SnowflakeValue, SnowflakeValueRef};
use crate::snowflake::{
    SnowflakeArguments, SnowflakeColumn, SnowflakeConnection, SnowflakeQueryResult, SnowflakeRow,
    SnowflakeStatement, SnowflakeTransactionManager, SnowflakeTypeInfo,
};

/// Snowflake database driver.
#[derive(Debug)]
pub struct Snowflake;

impl Database for Snowflake {
    type Connection = SnowflakeConnection;

    type TransactionManager = SnowflakeTransactionManager;

    type Row = SnowflakeRow;

    type QueryResult = SnowflakeQueryResult;

    type Column = SnowflakeColumn;

    type TypeInfo = SnowflakeTypeInfo;

    type Value = SnowflakeValue;
}

impl<'r> HasValueRef<'r> for Snowflake {
    type Database = Snowflake;

    type ValueRef = SnowflakeValueRef<'r>;
}

impl HasArguments<'_> for Snowflake {
    type Database = Snowflake;

    type Arguments = SnowflakeArguments;

    type ArgumentBuffer = SnowflakeArgumentBuffer;
}

impl<'q> HasStatement<'q> for Snowflake {
    type Database = Snowflake;

    type Statement = SnowflakeStatement<'q>;
}

impl HasStatementCache for Snowflake {}