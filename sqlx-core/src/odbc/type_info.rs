use crate::type_info::TypeInfo;
use odbc_api::DataType;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Type information for an ODBC type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdbcTypeInfo {
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

#[cfg(feature = "offline")]
#[derive(serde::Serialize, serde::Deserialize)]
enum SerializableDataType {
    Unknown,
    Char {
        length: Option<usize>,
    },
    WChar {
        length: Option<usize>,
    },
    Numeric {
        precision: usize,
        scale: i16,
    },
    Decimal {
        precision: usize,
        scale: i16,
    },
    Integer,
    SmallInt,
    Float {
        precision: usize,
    },
    Real,
    Double,
    Varchar {
        length: Option<usize>,
    },
    WVarchar {
        length: Option<usize>,
    },
    LongVarchar {
        length: Option<usize>,
    },
    WLongVarchar {
        length: Option<usize>,
    },
    LongVarbinary {
        length: Option<usize>,
    },
    Date,
    Time {
        precision: i16,
    },
    Timestamp {
        precision: i16,
    },
    BigInt,
    TinyInt,
    Bit,
    Varbinary {
        length: Option<usize>,
    },
    Binary {
        length: Option<usize>,
    },
    Other {
        data_type: i16,
        column_size: Option<usize>,
        decimal_digits: i16,
    },
}

#[cfg(feature = "offline")]
impl From<DataType> for SerializableDataType {
    fn from(data_type: DataType) -> Self {
        use SerializableDataType as Serializable;

        match data_type {
            DataType::Unknown => Serializable::Unknown,
            DataType::Char { length } => Serializable::Char {
                length: length.map(Into::into),
            },
            DataType::WChar { length } => Serializable::WChar {
                length: length.map(Into::into),
            },
            DataType::Numeric { precision, scale } => Serializable::Numeric { precision, scale },
            DataType::Decimal { precision, scale } => Serializable::Decimal { precision, scale },
            DataType::Integer => Serializable::Integer,
            DataType::SmallInt => Serializable::SmallInt,
            DataType::Float { precision } => Serializable::Float { precision },
            DataType::Real => Serializable::Real,
            DataType::Double => Serializable::Double,
            DataType::Varchar { length } => Serializable::Varchar {
                length: length.map(Into::into),
            },
            DataType::WVarchar { length } => Serializable::WVarchar {
                length: length.map(Into::into),
            },
            DataType::LongVarchar { length } => Serializable::LongVarchar {
                length: length.map(Into::into),
            },
            DataType::WLongVarchar { length } => Serializable::WLongVarchar {
                length: length.map(Into::into),
            },
            DataType::LongVarbinary { length } => Serializable::LongVarbinary {
                length: length.map(Into::into),
            },
            DataType::Date => Serializable::Date,
            DataType::Time { precision } => Serializable::Time { precision },
            DataType::Timestamp { precision } => Serializable::Timestamp { precision },
            DataType::BigInt => Serializable::BigInt,
            DataType::TinyInt => Serializable::TinyInt,
            DataType::Bit => Serializable::Bit,
            DataType::Varbinary { length } => Serializable::Varbinary {
                length: length.map(Into::into),
            },
            DataType::Binary { length } => Serializable::Binary {
                length: length.map(Into::into),
            },
            DataType::Other {
                data_type,
                column_size,
                decimal_digits,
            } => Serializable::Other {
                data_type: data_type.0,
                column_size: column_size.map(Into::into),
                decimal_digits,
            },
        }
    }
}

#[cfg(feature = "offline")]
impl From<SerializableDataType> for DataType {
    fn from(data_type: SerializableDataType) -> Self {
        use std::num::NonZeroUsize;
        use SerializableDataType as Serializable;

        match data_type {
            Serializable::Unknown => DataType::Unknown,
            Serializable::Char { length } => DataType::Char {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::WChar { length } => DataType::WChar {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::Numeric { precision, scale } => DataType::Numeric { precision, scale },
            Serializable::Decimal { precision, scale } => DataType::Decimal { precision, scale },
            Serializable::Integer => DataType::Integer,
            Serializable::SmallInt => DataType::SmallInt,
            Serializable::Float { precision } => DataType::Float { precision },
            Serializable::Real => DataType::Real,
            Serializable::Double => DataType::Double,
            Serializable::Varchar { length } => DataType::Varchar {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::WVarchar { length } => DataType::WVarchar {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::LongVarchar { length } => DataType::LongVarchar {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::WLongVarchar { length } => DataType::WLongVarchar {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::LongVarbinary { length } => DataType::LongVarbinary {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::Date => DataType::Date,
            Serializable::Time { precision } => DataType::Time { precision },
            Serializable::Timestamp { precision } => DataType::Timestamp { precision },
            Serializable::BigInt => DataType::BigInt,
            Serializable::TinyInt => DataType::TinyInt,
            Serializable::Bit => DataType::Bit,
            Serializable::Varbinary { length } => DataType::Varbinary {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::Binary { length } => DataType::Binary {
                length: length.and_then(NonZeroUsize::new),
            },
            Serializable::Other {
                data_type,
                column_size,
                decimal_digits,
            } => DataType::Other {
                data_type: odbc_api::sys::SqlDataType(data_type),
                column_size: column_size.and_then(NonZeroUsize::new),
                decimal_digits,
            },
        }
    }
}

#[cfg(feature = "offline")]
impl serde::Serialize for OdbcTypeInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableDataType::from(self.data_type).serialize(serializer)
    }
}

#[cfg(feature = "offline")]
impl<'de> serde::Deserialize<'de> for OdbcTypeInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(
            SerializableDataType::deserialize(deserializer)?.into(),
        ))
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

#[cfg(all(test, feature = "offline", feature = "json"))]
mod offline_tests {
    use super::*;
    use std::num::NonZeroUsize;

    #[test]
    fn offline_type_info_round_trips_losslessly() {
        let types = [
            DataType::Unknown,
            DataType::Char {
                length: NonZeroUsize::new(12),
            },
            DataType::WLongVarchar { length: None },
            DataType::Numeric {
                precision: 19,
                scale: 4,
            },
            DataType::Float { precision: 24 },
            DataType::Timestamp { precision: 6 },
            DataType::Varbinary {
                length: NonZeroUsize::new(128),
            },
            DataType::Other {
                data_type: odbc_api::sys::SqlDataType(-151),
                column_size: NonZeroUsize::new(36),
                decimal_digits: 2,
            },
        ];

        for data_type in types {
            let original = OdbcTypeInfo::new(data_type);
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: OdbcTypeInfo = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, original, "serialized metadata: {json}");
        }
    }
}
