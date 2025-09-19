use crate::type_info::TypeInfo;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdbcTypeInfo {
    pub(crate) name: String,
    pub(crate) is_null: bool,
}

impl TypeInfo for OdbcTypeInfo {
    fn is_null(&self) -> bool { self.is_null }
    fn name(&self) -> &str { &self.name }
}

impl Display for OdbcTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.name)
    }
}
