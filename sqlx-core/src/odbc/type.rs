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

impl Type<Odbc> for u8 {
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

impl Type<Odbc> for u16 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::SMALLINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::SmallInt | DataType::Integer | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::INTEGER
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Integer | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIGINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::BigInt | DataType::Integer | DataType::Numeric { .. } | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for &[u8] {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varbinary(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_binary_data() || ty.data_type().accepts_character_data()
        // Allow decoding from character types too
    }
}

// Feature-gated types
#[cfg(feature = "chrono")]
mod chrono_types {
    use super::*;
    use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};

    impl Type<Odbc> for NaiveDate {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::DATE
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            matches!(ty.data_type(), DataType::Date) || ty.data_type().accepts_character_data()
        }
    }

    impl Type<Odbc> for NaiveTime {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::TIME
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            matches!(ty.data_type(), DataType::Time { .. }) || ty.data_type().accepts_character_data()
        }
    }

    impl Type<Odbc> for NaiveDateTime {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::TIMESTAMP
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            matches!(ty.data_type(), DataType::Timestamp { .. }) || ty.data_type().accepts_character_data()
        }
    }

    impl Type<Odbc> for DateTime<Utc> {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::TIMESTAMP
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            matches!(ty.data_type(), DataType::Timestamp { .. }) || ty.data_type().accepts_character_data()
        }
    }

    impl Type<Odbc> for DateTime<FixedOffset> {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::TIMESTAMP
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            matches!(ty.data_type(), DataType::Timestamp { .. }) || ty.data_type().accepts_character_data()
        }
    }

    impl Type<Odbc> for DateTime<Local> {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::TIMESTAMP
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            matches!(ty.data_type(), DataType::Timestamp { .. }) || ty.data_type().accepts_character_data()
        }
    }
}

#[cfg(feature = "json")]
mod json_types {
    use super::*;
    use serde_json::Value;

    impl Type<Odbc> for Value {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::varchar(None)
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            ty.data_type().accepts_character_data()
        }
    }
}

#[cfg(feature = "bigdecimal")]
mod bigdecimal_types {
    use super::*;
    use bigdecimal::BigDecimal;

    impl Type<Odbc> for BigDecimal {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::numeric(28, 4) // Standard precision/scale
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            matches!(
                ty.data_type(),
                DataType::Numeric { .. } | DataType::Decimal { .. } | DataType::Double | DataType::Float { .. }
            ) || ty.data_type().accepts_character_data()
        }
    }
}

#[cfg(feature = "decimal")]
mod decimal_types {
    use super::*;
    use rust_decimal::Decimal;

    impl Type<Odbc> for Decimal {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::numeric(28, 4) // Standard precision/scale
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            matches!(
                ty.data_type(),
                DataType::Numeric { .. } | DataType::Decimal { .. } | DataType::Double | DataType::Float { .. }
            ) || ty.data_type().accepts_character_data()
        }
    }
}

#[cfg(feature = "uuid")]
mod uuid_types {
    use super::*;
    use uuid::Uuid;

    impl Type<Odbc> for Uuid {
        fn type_info() -> OdbcTypeInfo {
            OdbcTypeInfo::varchar(Some(std::num::NonZeroUsize::new(36).unwrap())) // UUID string length
        }
        fn compatible(ty: &OdbcTypeInfo) -> bool {
            ty.data_type().accepts_character_data() || ty.data_type().accepts_binary_data()
        }
    }
}

// Option<T> blanket impl is provided in core types; do not re-implement here.
