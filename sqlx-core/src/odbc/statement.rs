use crate::odbc::{Odbc, OdbcColumn, OdbcTypeInfo};
use crate::statement::Statement;
use crate::error::Error;
use crate::column::ColumnIndex;
use either::Either;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct OdbcStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) columns: Vec<OdbcColumn>,
    pub(crate) parameters: usize,
}

impl<'q> Statement<'q> for OdbcStatement<'q> {
    type Database = Odbc;

    fn to_owned(&self) -> OdbcStatement<'static> {
        OdbcStatement { sql: Cow::Owned(self.sql.to_string()), columns: self.columns.clone(), parameters: self.parameters }
    }

    fn sql(&self) -> &str { &self.sql }
    fn parameters(&self) -> Option<Either<&[OdbcTypeInfo], usize>> { Some(Either::Right(self.parameters)) }
    fn columns(&self) -> &[OdbcColumn] { &self.columns }

    // ODBC arguments placeholder
    impl_statement_query!(crate::odbc::OdbcArguments<'_>);
}

impl ColumnIndex<OdbcStatement<'_>> for &'_ str {
    fn index(&self, statement: &OdbcStatement<'_>) -> Result<usize, Error> {
        statement
            .columns
            .iter()
            .position(|c| c.name == *self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
    }
}
