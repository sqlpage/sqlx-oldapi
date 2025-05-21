use sqlx_core as sqlx;

impl_database_ext! {
    sqlx::mssql::Mssql {
        bool,
        i8,
        i16,
        i32,
        i64,
        f32,
        f64,
        String,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveTime,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveDate,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveDateTime,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::DateTime<sqlx::types::chrono::FixedOffset>,
    },
    ParamChecking::Weak,
    feature-types: _info => None,
    row = sqlx::mssql::MssqlRow,
    name = "MSSQL"
}
