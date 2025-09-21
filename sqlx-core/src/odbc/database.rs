use crate::database::{Database, HasArguments, HasStatement, HasStatementCache, HasValueRef};
use crate::odbc::{
    OdbcColumn, OdbcConnection, OdbcQueryResult, OdbcRow, OdbcStatement, OdbcTransactionManager,
    OdbcTypeInfo, OdbcValue, OdbcValueRef,
};

#[derive(Debug)]
pub struct Odbc;

impl Database for Odbc {
    type Connection = OdbcConnection;

    type TransactionManager = OdbcTransactionManager;

    type Row = OdbcRow;

    type QueryResult = OdbcQueryResult;

    type Column = OdbcColumn;

    type TypeInfo = OdbcTypeInfo;

    type Value = OdbcValue;
}

impl<'r> HasValueRef<'r> for Odbc {
    type Database = Odbc;

    type ValueRef = OdbcValueRef<'r>;
}

impl<'q> HasArguments<'q> for Odbc {
    type Database = Odbc;

    type Arguments = crate::odbc::OdbcArguments;

    type ArgumentBuffer = Vec<crate::odbc::OdbcArgumentValue>;
}

impl<'q> HasStatement<'q> for Odbc {
    type Database = Odbc;

    type Statement = OdbcStatement<'q>;
}

impl HasStatementCache for Odbc {}
