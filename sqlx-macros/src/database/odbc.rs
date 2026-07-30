use super::{DatabaseExt, ParamChecking};
use sqlx::odbc::{DataType, DataTypeExt};
use sqlx_core as sqlx;

impl DatabaseExt for sqlx::odbc::Odbc {
    const DATABASE_PATH: &'static str = "sqlx::odbc::Odbc";
    const ROW_PATH: &'static str = "sqlx::odbc::OdbcRow";
    const NAME: &'static str = "ODBC";
    const PARAM_CHECKING: ParamChecking = ParamChecking::Weak;

    fn param_type_for_id(info: &Self::TypeInfo) -> Option<&'static str> {
        match info.data_type() {
            data_type if data_type.accepts_character_data() => Some("&str"),
            data_type if data_type.accepts_binary_data() => Some("&[u8]"),
            _ => rust_type_for_id(info),
        }
    }

    fn return_type_for_id(info: &Self::TypeInfo) -> Option<&'static str> {
        rust_type_for_id(info)
    }

    fn get_feature_gate(info: &Self::TypeInfo) -> Option<&'static str> {
        match info.data_type() {
            DataType::Numeric { .. } | DataType::Decimal { .. } => {
                #[cfg(not(any(feature = "bigdecimal", feature = "decimal")))]
                return Some("bigdecimal");
            }
            DataType::Date | DataType::Time { .. } | DataType::Timestamp { .. } => {
                #[cfg(not(any(feature = "chrono", feature = "time")))]
                return Some("chrono");
            }
            _ => {}
        }

        None
    }
}

fn rust_type_for_id(info: &sqlx::odbc::OdbcTypeInfo) -> Option<&'static str> {
    match info.data_type() {
        DataType::Bit => Some("bool"),
        DataType::TinyInt => Some("i8"),
        DataType::SmallInt => Some("i16"),
        DataType::Integer => Some("i32"),
        DataType::BigInt => Some("i64"),
        DataType::Real => Some("f32"),
        DataType::Float { precision } if precision <= 24 => Some("f32"),
        DataType::Float { .. } | DataType::Double => Some("f64"),
        data_type if data_type.accepts_character_data() => Some("String"),
        data_type if data_type.accepts_binary_data() => Some("Vec<u8>"),

        DataType::Numeric { .. } | DataType::Decimal { .. } => decimal_type(),
        DataType::Date => date_type(),
        DataType::Time { .. } => time_type(),
        DataType::Timestamp { .. } => timestamp_type(),

        DataType::Unknown | DataType::Other { .. } => None,
        // Character and binary families are handled by the guarded arms above.
        _ => None,
    }
}

fn decimal_type() -> Option<&'static str> {
    #[cfg(feature = "bigdecimal")]
    return Some("sqlx::types::BigDecimal");

    #[cfg(all(not(feature = "bigdecimal"), feature = "decimal"))]
    return Some("sqlx::types::Decimal");

    #[cfg(not(any(feature = "bigdecimal", feature = "decimal")))]
    None
}

fn date_type() -> Option<&'static str> {
    #[cfg(feature = "chrono")]
    return Some("sqlx::types::chrono::NaiveDate");

    #[cfg(all(not(feature = "chrono"), feature = "time"))]
    return Some("sqlx::types::time::Date");

    #[cfg(not(any(feature = "chrono", feature = "time")))]
    None
}

fn time_type() -> Option<&'static str> {
    #[cfg(feature = "chrono")]
    return Some("sqlx::types::chrono::NaiveTime");

    #[cfg(all(not(feature = "chrono"), feature = "time"))]
    return Some("sqlx::types::time::Time");

    #[cfg(not(any(feature = "chrono", feature = "time")))]
    None
}

fn timestamp_type() -> Option<&'static str> {
    #[cfg(feature = "chrono")]
    return Some("sqlx::types::chrono::NaiveDateTime");

    #[cfg(all(not(feature = "chrono"), feature = "time"))]
    return Some("sqlx::types::time::PrimitiveDateTime");

    #[cfg(not(any(feature = "chrono", feature = "time")))]
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx_core::odbc::OdbcTypeInfo;

    #[test]
    fn maps_canonical_odbc_types() {
        assert_eq!(rust_type_for_id(&OdbcTypeInfo::BIT), Some("bool"));
        assert_eq!(rust_type_for_id(&OdbcTypeInfo::INTEGER), Some("i32"));
        assert_eq!(rust_type_for_id(&OdbcTypeInfo::BIGINT), Some("i64"));
        assert_eq!(rust_type_for_id(&OdbcTypeInfo::REAL), Some("f32"));
        assert_eq!(rust_type_for_id(&OdbcTypeInfo::DOUBLE), Some("f64"));
        assert_eq!(
            rust_type_for_id(&OdbcTypeInfo::varchar(None)),
            Some("String")
        );
        assert_eq!(
            rust_type_for_id(&OdbcTypeInfo::varbinary(None)),
            Some("Vec<u8>")
        );
        assert_eq!(rust_type_for_id(&OdbcTypeInfo::UNKNOWN), None);
    }
}
