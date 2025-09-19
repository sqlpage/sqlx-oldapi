use crate::odbc::Odbc;
use crate::types::Type;
use crate::odbc::OdbcTypeInfo;

impl Type<Odbc> for i32 {
    fn type_info() -> OdbcTypeInfo { OdbcTypeInfo { name: "INT".into(), is_null: false } }
    fn compatible(_ty: &OdbcTypeInfo) -> bool { true }
}

impl Type<Odbc> for i64 {
    fn type_info() -> OdbcTypeInfo { OdbcTypeInfo { name: "BIGINT".into(), is_null: false } }
    fn compatible(_ty: &OdbcTypeInfo) -> bool { true }
}

impl Type<Odbc> for f64 {
    fn type_info() -> OdbcTypeInfo { OdbcTypeInfo { name: "DOUBLE".into(), is_null: false } }
    fn compatible(_ty: &OdbcTypeInfo) -> bool { true }
}

impl Type<Odbc> for f32 {
    fn type_info() -> OdbcTypeInfo { OdbcTypeInfo { name: "FLOAT".into(), is_null: false } }
    fn compatible(_ty: &OdbcTypeInfo) -> bool { true }
}

impl Type<Odbc> for String {
    fn type_info() -> OdbcTypeInfo { OdbcTypeInfo { name: "TEXT".into(), is_null: false } }
    fn compatible(_ty: &OdbcTypeInfo) -> bool { true }
}

impl<'a> Type<Odbc> for &'a str {
    fn type_info() -> OdbcTypeInfo { OdbcTypeInfo { name: "TEXT".into(), is_null: false } }
    fn compatible(_ty: &OdbcTypeInfo) -> bool { true }
}

impl Type<Odbc> for Vec<u8> {
    fn type_info() -> OdbcTypeInfo { OdbcTypeInfo { name: "BLOB".into(), is_null: false } }
    fn compatible(_ty: &OdbcTypeInfo) -> bool { true }
}

// Option<T> blanket impl is provided in core types; do not re-implement here.
