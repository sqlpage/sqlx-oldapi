use crate::odbc::{Odbc, OdbcTypeInfo, OdbcDataType};
use crate::types::Type;

impl Type<Odbc> for i32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::INTEGER
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), OdbcDataType::Integer | OdbcDataType::SmallInt | OdbcDataType::TinyInt)
    }
}

impl Type<Odbc> for i64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIGINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), OdbcDataType::BigInt | OdbcDataType::Integer)
    }
}

impl Type<Odbc> for f64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::DOUBLE
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), OdbcDataType::Double | OdbcDataType::Float | OdbcDataType::Real)
    }
}

impl Type<Odbc> for f32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::FLOAT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), OdbcDataType::Float | OdbcDataType::Real)
    }
}

impl Type<Odbc> for String {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::VARCHAR
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().is_character_type()
    }
}

impl Type<Odbc> for &str {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::VARCHAR
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().is_character_type()
    }
}

impl Type<Odbc> for Vec<u8> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::VARBINARY
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().is_binary_type()
    }
}

impl Type<Odbc> for i16 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::SMALLINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), OdbcDataType::SmallInt | OdbcDataType::TinyInt)
    }
}

impl Type<Odbc> for i8 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TINYINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), OdbcDataType::TinyInt)
    }
}

impl Type<Odbc> for bool {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), OdbcDataType::Bit | OdbcDataType::TinyInt)
    }
}

// Option<T> blanket impl is provided in core types; do not re-implement here.

