use crate::column::Column;
use crate::odbc::{Odbc, OdbcTypeInfo};

#[derive(Debug, Clone)]
pub struct OdbcColumn {
    pub(crate) name: String,
    pub(crate) type_info: OdbcTypeInfo,
    pub(crate) ordinal: usize,
}

impl Column for OdbcColumn {
    type Database = Odbc;

    fn ordinal(&self) -> usize {
        self.ordinal
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn type_info(&self) -> &OdbcTypeInfo {
        &self.type_info
    }
}

mod private {
    use super::OdbcColumn;
    use crate::column::private_column::Sealed;
    impl Sealed for OdbcColumn {}
}
