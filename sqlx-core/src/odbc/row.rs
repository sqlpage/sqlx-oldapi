use crate::column::ColumnIndex;
use crate::database::HasValueRef;
use crate::error::Error;
use crate::odbc::{Odbc, OdbcColumn, OdbcTypeInfo, OdbcValueRef};
use crate::row::Row;

#[derive(Debug, Clone)]
pub struct OdbcRow {
    pub(crate) columns: Vec<OdbcColumn>,
    pub(crate) values: Vec<(OdbcTypeInfo, Option<Vec<u8>>)>,
}

impl Row for OdbcRow {
    type Database = Odbc;

    fn columns(&self) -> &[OdbcColumn] {
        &self.columns
    }

    fn try_get_raw<I>(
        &self,
        index: I,
    ) -> Result<<Self::Database as HasValueRef<'_>>::ValueRef, Error>
    where
        I: ColumnIndex<Self>,
    {
        let idx = index.index(self)?;
        let (ti, data) = &self.values[idx];
        Ok(OdbcValueRef {
            type_info: ti.clone(),
            is_null: data.is_none(),
            text: None,
            blob: data.as_deref(),
            int: None,
            float: None,
        })
    }
}

impl ColumnIndex<OdbcRow> for &str {
    fn index(&self, row: &OdbcRow) -> Result<usize, Error> {
        row.columns
            .iter()
            .position(|col| col.name == *self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
    }
}

mod private {
    use super::OdbcRow;
    use crate::row::private_row::Sealed;
    impl Sealed for OdbcRow {}
}

#[cfg(feature = "any")]
impl From<OdbcRow> for crate::any::AnyRow {
    fn from(row: OdbcRow) -> Self {
        let columns = row
            .columns
            .iter()
            .map(|col| crate::any::AnyColumn {
                kind: crate::any::column::AnyColumnKind::Odbc(col.clone()),
                type_info: crate::any::AnyTypeInfo::from(col.type_info.clone()),
            })
            .collect();

        crate::any::AnyRow {
            kind: crate::any::row::AnyRowKind::Odbc(row),
            columns,
        }
    }
}
