use crate::column::ColumnIndex;
use crate::error::Error;
use crate::odbc::{Odbc, OdbcColumn, OdbcTypeInfo};
use crate::statement::Statement;
use either::Either;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct OdbcStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) metadata: OdbcStatementMetadata,
}

#[derive(Debug, Clone)]
pub struct OdbcStatementMetadata {
    pub columns: Vec<OdbcColumn>,
    pub parameters: usize,
}

impl<'q> Statement<'q> for OdbcStatement<'q> {
    type Database = Odbc;

    fn to_owned(&self) -> OdbcStatement<'static> {
        OdbcStatement {
            sql: Cow::Owned(self.sql.to_string()),
            metadata: self.metadata.clone(),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }
    fn parameters(&self) -> Option<Either<&[OdbcTypeInfo], usize>> {
        Some(Either::Right(self.metadata.parameters))
    }
    fn columns(&self) -> &[OdbcColumn] {
        &self.metadata.columns
    }

    // ODBC arguments placeholder
    impl_statement_query!(crate::odbc::OdbcArguments);
}

impl ColumnIndex<OdbcStatement<'_>> for &'_ str {
    fn index(&self, statement: &OdbcStatement<'_>) -> Result<usize, Error> {
        statement
            .metadata
            .columns
            .iter()
            .position(|c| c.name == *self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
    }
}

#[cfg(feature = "any")]
impl<'q> From<OdbcStatement<'q>> for crate::any::AnyStatement<'q> {
    fn from(stmt: OdbcStatement<'q>) -> Self {
        let mut column_names = crate::HashMap::<crate::ext::ustr::UStr, usize>::default();

        // First build the columns and collect names
        let columns: Vec<_> = stmt
            .metadata
            .columns
            .iter()
            .enumerate()
            .map(|(index, col)| {
                column_names.insert(crate::ext::ustr::UStr::new(&col.name), index);
                crate::any::AnyColumn {
                    kind: crate::any::column::AnyColumnKind::Odbc(col.clone()),
                    type_info: crate::any::AnyTypeInfo::from(col.type_info.clone()),
                }
            })
            .collect();

        crate::any::AnyStatement {
            sql: stmt.sql,
            parameters: Some(either::Either::Right(stmt.metadata.parameters)),
            columns,
            column_names: std::sync::Arc::new(column_names),
        }
    }
}
