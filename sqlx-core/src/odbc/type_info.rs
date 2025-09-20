use crate::type_info::TypeInfo;
use odbc_api::DataType;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Type information for an ODBC type.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct OdbcTypeInfo {
    #[cfg_attr(feature = "offline", serde(skip))]
    pub(crate) data_type: DataType,
}

impl OdbcTypeInfo {
    /// Create a new OdbcTypeInfo with the given data type
    pub const fn new(data_type: DataType) -> Self {
        Self { data_type }
    }

    /// Get the underlying data type
    pub const fn data_type(&self) -> DataType {
        self.data_type
    }
}

/// Extension trait for DataType with helper methods
pub trait DataTypeExt {
    /// Get the display name for this data type
    fn name(self) -> &'static str;

    /// Check if this is a character/string type
    fn accepts_character_data(self) -> bool;

    /// Check if this is a binary type
    fn accepts_binary_data(self) -> bool;

    /// Check if this is a numeric type
    fn accepts_numeric_data(self) -> bool;

    /// Check if this is a date/time type
    fn accepts_datetime_data(self) -> bool;
}

impl DataTypeExt for DataType {
    fn name(self) -> &'static str {
        match self {
            DataType::BigInt => "BIGINT",
            DataType::Binary { .. } => "BINARY",
            DataType::Bit => "BIT",
            DataType::Char { .. } => "CHAR",
            DataType::Date => "DATE",
            DataType::Decimal { .. } => "DECIMAL",
            DataType::Double => "DOUBLE",
            DataType::Float { .. } => "FLOAT",
            DataType::Integer => "INTEGER",
            DataType::LongVarbinary { .. } => "LONGVARBINARY",
            DataType::LongVarchar { .. } => "LONGVARCHAR",
            DataType::Numeric { .. } => "NUMERIC",
            DataType::Real => "REAL",
            DataType::SmallInt => "SMALLINT",
            DataType::Time { .. } => "TIME",
            DataType::Timestamp { .. } => "TIMESTAMP",
            DataType::TinyInt => "TINYINT",
            DataType::Varbinary { .. } => "VARBINARY",
            DataType::Varchar { .. } => "VARCHAR",
            DataType::WChar { .. } => "WCHAR",
            DataType::WLongVarchar { .. } => "WLONGVARCHAR",
            DataType::WVarchar { .. } => "WVARCHAR",
            DataType::Unknown => "UNKNOWN",
            DataType::Other { .. } => "OTHER",
        }
    }

    fn accepts_character_data(self) -> bool {
        matches!(
            self,
            DataType::Char { .. }
                | DataType::Varchar { .. }
                | DataType::LongVarchar { .. }
                | DataType::WChar { .. }
                | DataType::WVarchar { .. }
                | DataType::WLongVarchar { .. }
        )
    }

    fn accepts_binary_data(self) -> bool {
        matches!(
            self,
            DataType::Binary { .. } | DataType::Varbinary { .. } | DataType::LongVarbinary { .. }
        )
    }

    fn accepts_numeric_data(self) -> bool {
        matches!(
            self,
            DataType::TinyInt
                | DataType::SmallInt
                | DataType::Integer
                | DataType::BigInt
                | DataType::Real
                | DataType::Float { .. }
                | DataType::Double
                | DataType::Decimal { .. }
                | DataType::Numeric { .. }
        )
    }

    fn accepts_datetime_data(self) -> bool {
        matches!(
            self,
            DataType::Date | DataType::Time { .. } | DataType::Timestamp { .. }
        )
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
    pub const BIGINT: Self = Self::new(DataType::BigInt);
    pub const BIT: Self = Self::new(DataType::Bit);
    pub const DATE: Self = Self::new(DataType::Date);
    pub const DOUBLE: Self = Self::new(DataType::Double);
    pub const INTEGER: Self = Self::new(DataType::Integer);
    pub const REAL: Self = Self::new(DataType::Real);
    pub const SMALLINT: Self = Self::new(DataType::SmallInt);
    pub const TINYINT: Self = Self::new(DataType::TinyInt);
    pub const UNKNOWN: Self = Self::new(DataType::Unknown);
    pub const TIME: Self = Self::new(DataType::Time { precision: 0 });
    pub const TIMESTAMP: Self = Self::new(DataType::Timestamp { precision: 0 });

    // For types with parameters, use constructor functions
    pub const fn varchar(length: Option<std::num::NonZeroUsize>) -> Self {
        Self::new(DataType::Varchar { length })
    }

    pub const fn varbinary(length: Option<std::num::NonZeroUsize>) -> Self {
        Self::new(DataType::Varbinary { length })
    }

    pub const fn char(length: Option<std::num::NonZeroUsize>) -> Self {
        Self::new(DataType::Char { length })
    }

    pub const fn binary(length: Option<std::num::NonZeroUsize>) -> Self {
        Self::new(DataType::Binary { length })
    }

    pub const fn float(precision: usize) -> Self {
        Self::new(DataType::Float { precision })
    }

    pub const fn decimal(precision: usize, scale: i16) -> Self {
        Self::new(DataType::Decimal { precision, scale })
    }

    pub const fn numeric(precision: usize, scale: i16) -> Self {
        Self::new(DataType::Numeric { precision, scale })
    }

    pub const fn time(precision: i16) -> Self {
        Self::new(DataType::Time { precision })
    }

    pub const fn timestamp(precision: i16) -> Self {
        Self::new(DataType::Timestamp { precision })
    }
}

#[cfg(feature = "any")]
impl From<OdbcTypeInfo> for crate::any::AnyTypeInfo {
    fn from(info: OdbcTypeInfo) -> Self {
        crate::any::AnyTypeInfo(crate::any::type_info::AnyTypeInfoKind::Odbc(info))
    }
}
