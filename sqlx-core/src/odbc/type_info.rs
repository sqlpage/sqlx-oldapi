use crate::type_info::TypeInfo;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// ODBC data type enum based on the ODBC API DataType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub enum OdbcDataType {
    BigInt,
    Binary,
    Bit,
    Char,
    Date,
    Decimal,
    Double,
    Float,
    Integer,
    LongVarbinary,
    LongVarchar,
    Numeric,
    Real,
    SmallInt,
    Time,
    Timestamp,
    TinyInt,
    Varbinary,
    Varchar,
    WChar,
    WLongVarchar,
    WVarchar,
    Unknown,
}

/// Type information for an ODBC type.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct OdbcTypeInfo {
    pub(crate) data_type: OdbcDataType,
    pub(crate) precision: Option<u32>,
    pub(crate) scale: Option<u16>,
    pub(crate) length: Option<u32>,
}

impl OdbcTypeInfo {
    /// Create a new OdbcTypeInfo with the given data type
    pub const fn new(data_type: OdbcDataType) -> Self {
        Self {
            data_type,
            precision: None,
            scale: None,
            length: None,
        }
    }

    /// Create a new OdbcTypeInfo with precision
    pub const fn with_precision(data_type: OdbcDataType, precision: u32) -> Self {
        Self {
            data_type,
            precision: Some(precision),
            scale: None,
            length: None,
        }
    }

    /// Create a new OdbcTypeInfo with precision and scale
    pub const fn with_precision_and_scale(data_type: OdbcDataType, precision: u32, scale: u16) -> Self {
        Self {
            data_type,
            precision: Some(precision),
            scale: Some(scale),
            length: None,
        }
    }

    /// Create a new OdbcTypeInfo with length
    pub const fn with_length(data_type: OdbcDataType, length: u32) -> Self {
        Self {
            data_type,
            precision: None,
            scale: None,
            length: Some(length),
        }
    }

    /// Get the underlying data type
    pub const fn data_type(&self) -> OdbcDataType {
        self.data_type
    }

    /// Get the precision if any
    pub const fn precision(&self) -> Option<u32> {
        self.precision
    }

    /// Get the scale if any
    pub const fn scale(&self) -> Option<u16> {
        self.scale
    }

    /// Get the length if any
    pub const fn length(&self) -> Option<u32> {
        self.length
    }
}

impl OdbcDataType {
    /// Get the display name for this data type
    pub const fn name(self) -> &'static str {
        match self {
            OdbcDataType::BigInt => "BIGINT",
            OdbcDataType::Binary => "BINARY",
            OdbcDataType::Bit => "BIT",
            OdbcDataType::Char => "CHAR",
            OdbcDataType::Date => "DATE",
            OdbcDataType::Decimal => "DECIMAL",
            OdbcDataType::Double => "DOUBLE",
            OdbcDataType::Float => "FLOAT",
            OdbcDataType::Integer => "INTEGER",
            OdbcDataType::LongVarbinary => "LONGVARBINARY",
            OdbcDataType::LongVarchar => "LONGVARCHAR",
            OdbcDataType::Numeric => "NUMERIC",
            OdbcDataType::Real => "REAL",
            OdbcDataType::SmallInt => "SMALLINT",
            OdbcDataType::Time => "TIME",
            OdbcDataType::Timestamp => "TIMESTAMP",
            OdbcDataType::TinyInt => "TINYINT",
            OdbcDataType::Varbinary => "VARBINARY",
            OdbcDataType::Varchar => "VARCHAR",
            OdbcDataType::WChar => "WCHAR",
            OdbcDataType::WLongVarchar => "WLONGVARCHAR",
            OdbcDataType::WVarchar => "WVARCHAR",
            OdbcDataType::Unknown => "UNKNOWN",
        }
    }

    /// Check if this is a character/string type
    pub const fn is_character_type(self) -> bool {
        matches!(self, OdbcDataType::Char | OdbcDataType::Varchar | OdbcDataType::LongVarchar |
                      OdbcDataType::WChar | OdbcDataType::WVarchar | OdbcDataType::WLongVarchar)
    }

    /// Check if this is a binary type
    pub const fn is_binary_type(self) -> bool {
        matches!(self, OdbcDataType::Binary | OdbcDataType::Varbinary | OdbcDataType::LongVarbinary)
    }

    /// Check if this is a numeric type
    pub const fn is_numeric_type(self) -> bool {
        matches!(self, OdbcDataType::TinyInt | OdbcDataType::SmallInt | OdbcDataType::Integer |
                      OdbcDataType::BigInt | OdbcDataType::Real | OdbcDataType::Float |
                      OdbcDataType::Double | OdbcDataType::Decimal | OdbcDataType::Numeric)
    }

    /// Check if this is a date/time type
    pub const fn is_datetime_type(self) -> bool {
        matches!(self, OdbcDataType::Date | OdbcDataType::Time | OdbcDataType::Timestamp)
    }
}

impl TypeInfo for OdbcTypeInfo {
    fn is_null(&self) -> bool {
        false
    }
    
    fn name(&self) -> &str {
        self.data_type.name()
    }

    fn is_void(&self) -> bool {
        false
    }
}

impl Display for OdbcTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

// Provide some common type constants
impl OdbcTypeInfo {
    pub const BIGINT: Self = Self::new(OdbcDataType::BigInt);
    pub const BINARY: Self = Self::new(OdbcDataType::Binary);
    pub const BIT: Self = Self::new(OdbcDataType::Bit);
    pub const CHAR: Self = Self::new(OdbcDataType::Char);
    pub const DATE: Self = Self::new(OdbcDataType::Date);
    pub const DECIMAL: Self = Self::new(OdbcDataType::Decimal);
    pub const DOUBLE: Self = Self::new(OdbcDataType::Double);
    pub const FLOAT: Self = Self::new(OdbcDataType::Float);
    pub const INTEGER: Self = Self::new(OdbcDataType::Integer);
    pub const LONGVARBINARY: Self = Self::new(OdbcDataType::LongVarbinary);
    pub const LONGVARCHAR: Self = Self::new(OdbcDataType::LongVarchar);
    pub const NUMERIC: Self = Self::new(OdbcDataType::Numeric);
    pub const REAL: Self = Self::new(OdbcDataType::Real);
    pub const SMALLINT: Self = Self::new(OdbcDataType::SmallInt);
    pub const TIME: Self = Self::new(OdbcDataType::Time);
    pub const TIMESTAMP: Self = Self::new(OdbcDataType::Timestamp);
    pub const TINYINT: Self = Self::new(OdbcDataType::TinyInt);
    pub const VARBINARY: Self = Self::new(OdbcDataType::Varbinary);
    pub const VARCHAR: Self = Self::new(OdbcDataType::Varchar);
    pub const WCHAR: Self = Self::new(OdbcDataType::WChar);
    pub const WLONGVARCHAR: Self = Self::new(OdbcDataType::WLongVarchar);
    pub const WVARCHAR: Self = Self::new(OdbcDataType::WVarchar);
    pub const UNKNOWN: Self = Self::new(OdbcDataType::Unknown);
}
