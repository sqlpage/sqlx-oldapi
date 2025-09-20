use crate::odbc::{DataTypeExt, Odbc, OdbcTypeInfo};
use crate::types::Type;
use odbc_api::DataType;

impl Type<Odbc> for i32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::INTEGER
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Integer | DataType::SmallInt | DataType::TinyInt | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for i64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIGINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::BigInt
                | DataType::Integer
                | DataType::SmallInt
                | DataType::TinyInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for f64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::DOUBLE
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Double
                | DataType::Float { .. }
                | DataType::Real
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
                | DataType::Integer
                | DataType::BigInt
                | DataType::SmallInt
                | DataType::TinyInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for f32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::float(24) // Standard float precision
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Float { .. }
                | DataType::Real
                | DataType::Double
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
                | DataType::Integer
                | DataType::BigInt
                | DataType::SmallInt
                | DataType::TinyInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for String {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varchar(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for &str {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varchar(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for Vec<u8> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varbinary(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_binary_data() || ty.data_type().accepts_character_data()
        // Allow decoding from character types too
    }
}

impl Type<Odbc> for i16 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::SMALLINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::SmallInt | DataType::TinyInt | DataType::Integer | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for i8 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TINYINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::TinyInt | DataType::SmallInt | DataType::Integer | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for bool {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Bit | DataType::TinyInt | DataType::SmallInt | DataType::Integer
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

// Option<T> blanket impl is provided in core types; do not re-implement here.
