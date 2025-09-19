use crate::type_info::TypeInfo;
use std::fmt::{self, Display};

/// Type information for Snowflake.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SnowflakeTypeInfo(pub(crate) SnowflakeType);

/// The Snowflake data types.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SnowflakeType {
    // Numeric types
    Number,
    Decimal,
    Numeric,
    Int,
    Integer,
    Bigint,
    Smallint,
    Tinyint,
    Byteint,
    Float,
    Float4,
    Float8,
    Double,
    DoublePrecision,
    Real,

    // String types
    Varchar,
    Char,
    Character,
    String,
    Text,

    // Binary types
    Binary,
    Varbinary,

    // Boolean type
    Boolean,

    // Date/Time types
    Date,
    Datetime,
    Time,
    Timestamp,
    TimestampLtz,
    TimestampNtz,
    TimestampTz,

    // Semi-structured types
    Variant,
    Object,
    Array,

    // Geography type
    Geography,
    Geometry,
}

impl SnowflakeTypeInfo {
    pub fn new(ty: SnowflakeType) -> Self {
        Self(ty)
    }

    pub fn r#type(&self) -> &SnowflakeType {
        &self.0
    }

    pub fn name(&self) -> &str {
        self.0.name()
    }
}

impl Display for SnowflakeTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl TypeInfo for SnowflakeTypeInfo {
    fn is_null(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        self.0.name()
    }
}

impl SnowflakeType {
    pub fn name(&self) -> &str {
        match self {
            SnowflakeType::Number => "NUMBER",
            SnowflakeType::Decimal => "DECIMAL",
            SnowflakeType::Numeric => "NUMERIC",
            SnowflakeType::Int => "INT",
            SnowflakeType::Integer => "INTEGER",
            SnowflakeType::Bigint => "BIGINT",
            SnowflakeType::Smallint => "SMALLINT",
            SnowflakeType::Tinyint => "TINYINT",
            SnowflakeType::Byteint => "BYTEINT",
            SnowflakeType::Float => "FLOAT",
            SnowflakeType::Float4 => "FLOAT4",
            SnowflakeType::Float8 => "FLOAT8",
            SnowflakeType::Double => "DOUBLE",
            SnowflakeType::DoublePrecision => "DOUBLE PRECISION",
            SnowflakeType::Real => "REAL",
            SnowflakeType::Varchar => "VARCHAR",
            SnowflakeType::Char => "CHAR",
            SnowflakeType::Character => "CHARACTER",
            SnowflakeType::String => "STRING",
            SnowflakeType::Text => "TEXT",
            SnowflakeType::Binary => "BINARY",
            SnowflakeType::Varbinary => "VARBINARY",
            SnowflakeType::Boolean => "BOOLEAN",
            SnowflakeType::Date => "DATE",
            SnowflakeType::Datetime => "DATETIME",
            SnowflakeType::Time => "TIME",
            SnowflakeType::Timestamp => "TIMESTAMP",
            SnowflakeType::TimestampLtz => "TIMESTAMP_LTZ",
            SnowflakeType::TimestampNtz => "TIMESTAMP_NTZ",
            SnowflakeType::TimestampTz => "TIMESTAMP_TZ",
            SnowflakeType::Variant => "VARIANT",
            SnowflakeType::Object => "OBJECT",
            SnowflakeType::Array => "ARRAY",
            SnowflakeType::Geography => "GEOGRAPHY",
            SnowflakeType::Geometry => "GEOMETRY",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "NUMBER" => Some(SnowflakeType::Number),
            "DECIMAL" => Some(SnowflakeType::Decimal),
            "NUMERIC" => Some(SnowflakeType::Numeric),
            "INT" => Some(SnowflakeType::Int),
            "INTEGER" => Some(SnowflakeType::Integer),
            "BIGINT" => Some(SnowflakeType::Bigint),
            "SMALLINT" => Some(SnowflakeType::Smallint),
            "TINYINT" => Some(SnowflakeType::Tinyint),
            "BYTEINT" => Some(SnowflakeType::Byteint),
            "FLOAT" => Some(SnowflakeType::Float),
            "FLOAT4" => Some(SnowflakeType::Float4),
            "FLOAT8" => Some(SnowflakeType::Float8),
            "DOUBLE" => Some(SnowflakeType::Double),
            "DOUBLE PRECISION" => Some(SnowflakeType::DoublePrecision),
            "REAL" => Some(SnowflakeType::Real),
            "VARCHAR" => Some(SnowflakeType::Varchar),
            "CHAR" => Some(SnowflakeType::Char),
            "CHARACTER" => Some(SnowflakeType::Character),
            "STRING" => Some(SnowflakeType::String),
            "TEXT" => Some(SnowflakeType::Text),
            "BINARY" => Some(SnowflakeType::Binary),
            "VARBINARY" => Some(SnowflakeType::Varbinary),
            "BOOLEAN" => Some(SnowflakeType::Boolean),
            "DATE" => Some(SnowflakeType::Date),
            "DATETIME" => Some(SnowflakeType::Datetime),
            "TIME" => Some(SnowflakeType::Time),
            "TIMESTAMP" => Some(SnowflakeType::Timestamp),
            "TIMESTAMP_LTZ" => Some(SnowflakeType::TimestampLtz),
            "TIMESTAMP_NTZ" => Some(SnowflakeType::TimestampNtz),
            "TIMESTAMP_TZ" => Some(SnowflakeType::TimestampTz),
            "VARIANT" => Some(SnowflakeType::Variant),
            "OBJECT" => Some(SnowflakeType::Object),
            "ARRAY" => Some(SnowflakeType::Array),
            "GEOGRAPHY" => Some(SnowflakeType::Geography),
            "GEOMETRY" => Some(SnowflakeType::Geometry),
            _ => None,
        }
    }
}

#[cfg(all(feature = "any", any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite")))]
impl From<SnowflakeTypeInfo> for crate::any::AnyTypeInfo {
    #[inline]
    fn from(ty: SnowflakeTypeInfo) -> Self {
        crate::any::AnyTypeInfo(crate::any::type_info::AnyTypeInfoKind::Snowflake(ty))
    }
}
