use crate::column::ColumnIndex;
use crate::database::HasValueRef;
use crate::error::Error;
use crate::odbc::{Odbc, OdbcColumn, OdbcValueRef};
use crate::row::Row;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct OdbcBatch {
    pub(crate) columns: Arc<[OdbcColumn]>,
    pub(crate) column_data: Vec<Arc<crate::odbc::ColumnData>>,
}

#[derive(Debug, Clone)]
pub struct OdbcRow {
    pub(crate) row_index: usize,
    pub(crate) batch: Arc<OdbcBatch>,
}

impl Row for OdbcRow {
    type Database = Odbc;

    fn columns(&self) -> &[OdbcColumn] {
        &self.batch.columns
    }

    fn try_get_raw<I>(
        &self,
        index: I,
    ) -> Result<<Self::Database as HasValueRef<'_>>::ValueRef, Error>
    where
        I: ColumnIndex<Self>,
    {
        let column_index = index.index(self)?;
        Ok(OdbcValueRef::new(&self.batch, self.row_index, column_index))
    }
}

impl ColumnIndex<OdbcRow> for &str {
    fn index(&self, row: &OdbcRow) -> Result<usize, Error> {
        // Try exact match first (for performance)
        if let Some(pos) = row.batch.columns.iter().position(|col| col.name == *self) {
            return Ok(pos);
        }

        // Fall back to case-insensitive match (for databases like Snowflake)
        row.batch
            .columns
            .iter()
            .position(|col| col.name.eq_ignore_ascii_case(self))
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
            .batch
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odbc::{ColumnData, OdbcColumn, OdbcTypeInfo, OdbcValueVec};
    use crate::type_info::TypeInfo;
    use crate::value::ValueRef;
    use odbc_api::DataType;
    use std::sync::Arc;

    fn create_test_row() -> OdbcRow {
        let columns = Arc::new([
            OdbcColumn {
                name: "lowercase_col".to_string(),
                type_info: OdbcTypeInfo::new(DataType::Integer),
                ordinal: 0,
            },
            OdbcColumn {
                name: "UPPERCASE_COL".to_string(),
                type_info: OdbcTypeInfo::new(DataType::Varchar { length: None }),
                ordinal: 1,
            },
            OdbcColumn {
                name: "MixedCase_Col".to_string(),
                type_info: OdbcTypeInfo::new(DataType::Double),
                ordinal: 2,
            },
        ]);

        let column_data = vec![
            Arc::new(ColumnData {
                values: OdbcValueVec::BigInt(vec![42]),
                type_info: OdbcTypeInfo::new(DataType::Integer),
                nulls: vec![false],
            }),
            Arc::new(ColumnData {
                values: OdbcValueVec::Text(vec!["test".to_string()]),
                type_info: OdbcTypeInfo::new(DataType::Varchar { length: None }),
                nulls: vec![false],
            }),
            Arc::new(ColumnData {
                values: OdbcValueVec::Double(vec![std::f64::consts::PI]),
                type_info: OdbcTypeInfo::new(DataType::Double),
                nulls: vec![false],
            }),
        ];

        OdbcRow {
            row_index: 0,
            batch: Arc::new(OdbcBatch {
                columns,
                column_data,
            }),
        }
    }

    #[test]
    fn test_exact_column_match() {
        let row = create_test_row();

        // Exact matches should work
        assert_eq!("lowercase_col".index(&row).unwrap(), 0);
        assert_eq!("UPPERCASE_COL".index(&row).unwrap(), 1);
        assert_eq!("MixedCase_Col".index(&row).unwrap(), 2);
    }

    #[test]
    fn test_case_insensitive_column_match() {
        let row = create_test_row();

        // Case-insensitive matches should work
        assert_eq!("LOWERCASE_COL".index(&row).unwrap(), 0);
        assert_eq!("lowercase_col".index(&row).unwrap(), 0);
        assert_eq!("uppercase_col".index(&row).unwrap(), 1);
        assert_eq!("UPPERCASE_COL".index(&row).unwrap(), 1);
        assert_eq!("mixedcase_col".index(&row).unwrap(), 2);
        assert_eq!("MIXEDCASE_COL".index(&row).unwrap(), 2);
        assert_eq!("MixedCase_Col".index(&row).unwrap(), 2);
    }

    #[test]
    fn test_column_not_found() {
        let row = create_test_row();

        let result = "nonexistent_column".index(&row);
        assert!(result.is_err());
        if let Err(Error::ColumnNotFound(name)) = result {
            assert_eq!(name, "nonexistent_column");
        } else {
            panic!("Expected ColumnNotFound error");
        }
    }

    #[test]
    fn test_try_get_raw() {
        let row = create_test_row();

        // Test accessing by exact name
        let value = row.try_get_raw("lowercase_col").unwrap();
        assert!(!value.is_null());
        assert_eq!(value.type_info().name(), "INTEGER");

        // Test accessing by case-insensitive name
        let value = row.try_get_raw("LOWERCASE_COL").unwrap();
        assert!(!value.is_null());
        assert_eq!(value.type_info().name(), "INTEGER");

        // Test accessing uppercase column with lowercase name
        let value = row.try_get_raw("uppercase_col").unwrap();
        assert!(!value.is_null());
        assert_eq!(value.type_info().name(), "VARCHAR");
    }

    #[test]
    fn test_columns_method() {
        let row = create_test_row();
        let columns = row.columns();

        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].name, "lowercase_col");
        assert_eq!(columns[1].name, "UPPERCASE_COL");
        assert_eq!(columns[2].name, "MixedCase_Col");
    }
}
