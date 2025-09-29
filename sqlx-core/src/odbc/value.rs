use crate::odbc::{Odbc, OdbcTypeInfo};
use crate::value::{Value, ValueRef};
use odbc_api::buffers::AnySlice;
use odbc_api::sys::NULL_DATA;
use std::borrow::Cow;
use std::sync::Arc;

/// Enum containing owned column data for all supported ODBC types
#[derive(Debug, Clone)]
pub enum OdbcValueVec {
    // Integer types
    TinyInt(Vec<i8>),
    SmallInt(Vec<i16>),
    Integer(Vec<i32>),
    BigInt(Vec<i64>),

    // Floating point types
    Real(Vec<f32>),
    Double(Vec<f64>),

    // Bit type
    Bit(Vec<odbc_api::Bit>),

    // Text types (inherently nullable in ODBC)
    Text(Vec<Option<String>>),

    // Binary types (inherently nullable in ODBC)
    Binary(Vec<Option<Vec<u8>>>),

    // Date/Time types
    Date(Vec<odbc_api::sys::Date>),
    Time(Vec<odbc_api::sys::Time>),
    Timestamp(Vec<odbc_api::sys::Timestamp>),
}

/// Container for column data with type information
#[derive(Debug, Clone)]
pub struct ColumnData {
    pub values: OdbcValueVec,
    pub type_info: OdbcTypeInfo,
    pub nulls: Vec<bool>,
}

#[derive(Debug)]
pub struct OdbcValueRef<'r> {
    pub(crate) column_data: &'r ColumnData,
    pub(crate) row_index: usize,
}

#[derive(Debug, Clone)]
pub struct OdbcValue {
    pub(crate) column_data: Arc<ColumnData>,
    pub(crate) row_index: usize,
}

impl<'r> ValueRef<'r> for OdbcValueRef<'r> {
    type Database = Odbc;

    fn to_owned(&self) -> OdbcValue {
        OdbcValue {
            column_data: Arc::new(self.column_data.clone()),
            row_index: self.row_index,
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.column_data.type_info)
    }

    fn is_null(&self) -> bool {
        value_vec_is_null(self.column_data, self.row_index)
    }
}

impl Value for OdbcValue {
    type Database = Odbc;

    fn as_ref(&self) -> OdbcValueRef<'_> {
        OdbcValueRef {
            column_data: &self.column_data,
            row_index: self.row_index,
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.column_data.type_info)
    }

    fn is_null(&self) -> bool {
        value_vec_is_null(&self.column_data, self.row_index)
    }
}

/// Utility methods for OdbcValue
impl OdbcValue {
    /// Create a new OdbcValue from column data and row index
    pub fn new(column_data: Arc<ColumnData>, row_index: usize) -> Self {
        Self {
            column_data,
            row_index,
        }
    }

    /// Get the raw value from the column data
    pub fn get_raw(&self) -> Option<OdbcValueType> {
        value_vec_get_raw(&self.column_data.values, self.row_index)
    }

    /// Try to get the value as i64
    pub fn as_int<T: TryFromInt>(&self) -> Option<T> {
        value_vec_int(&self.column_data.values, self.row_index)
    }

    /// Try to get the value as f64
    pub fn as_f64(&self) -> Option<f64> {
        value_vec_float(&self.column_data.values, self.row_index)
    }

    /// Try to get the value as string
    pub fn as_str(&self) -> Option<Cow<'_, str>> {
        value_vec_text(&self.column_data.values, self.row_index).map(Cow::Borrowed)
    }

    /// Try to get the value as bytes
    pub fn as_bytes(&self) -> Option<Cow<'_, [u8]>> {
        value_vec_blob(&self.column_data.values, self.row_index).map(Cow::Borrowed)
    }
}

/// Utility methods for OdbcValueRef
impl<'r> OdbcValueRef<'r> {
    /// Create a new OdbcValueRef from column data and row index
    pub fn new(column_data: &'r ColumnData, row_index: usize) -> Self {
        Self {
            column_data,
            row_index,
        }
    }

    /// Get the raw value from the column data
    pub fn get_raw(&self) -> Option<OdbcValueType> {
        value_vec_get_raw(&self.column_data.values, self.row_index)
    }

    /// Try to get the value as i64
    pub fn int<T: TryFromInt>(&self) -> Option<T> {
        value_vec_int(&self.column_data.values, self.row_index)
    }

    pub fn try_int<T: TryFromInt + crate::types::Type<Odbc>>(&self) -> crate::error::Result<T> {
        if !T::compatible(&self.column_data.type_info) {
            return Err(crate::error::Error::Decode(
                crate::error::mismatched_types::<Odbc, T>(&self.column_data.type_info),
            ));
        }
        self.int::<T>().ok_or_else(|| {
            crate::error::Error::Decode(crate::error::mismatched_types::<Odbc, T>(
                &self.column_data.type_info,
            ))
        })
    }

    pub fn try_float<T: TryFromFloat + crate::types::Type<Odbc>>(&self) -> crate::error::Result<T> {
        if !T::compatible(&self.column_data.type_info) {
            return Err(crate::error::Error::Decode(
                crate::error::mismatched_types::<Odbc, T>(&self.column_data.type_info),
            ));
        }
        self.float::<T>().ok_or_else(|| {
            crate::error::Error::Decode(crate::error::mismatched_types::<Odbc, T>(
                &self.column_data.type_info,
            ))
        })
    }

    /// Try to get the value as f64
    pub fn float<T: TryFromFloat>(&self) -> Option<T> {
        value_vec_float(&self.column_data.values, self.row_index)
    }

    /// Try to get the value as string slice
    pub fn text(&self) -> Option<&'r str> {
        value_vec_text(&self.column_data.values, self.row_index)
    }

    /// Try to get the value as binary slice
    pub fn blob(&self) -> Option<&'r [u8]> {
        value_vec_blob(&self.column_data.values, self.row_index)
    }

    /// Try to get the raw ODBC Date value
    pub fn date(&self) -> Option<odbc_api::sys::Date> {
        if self.is_null() {
            None
        } else {
            match &self.column_data.values {
                OdbcValueVec::Date(raw_values) => raw_values.get(self.row_index).copied(),
                _ => None,
            }
        }
    }

    /// Try to get the raw ODBC Time value
    pub fn time(&self) -> Option<odbc_api::sys::Time> {
        if self.is_null() {
            None
        } else {
            match &self.column_data.values {
                OdbcValueVec::Time(raw_values) => raw_values.get(self.row_index).copied(),
                _ => None,
            }
        }
    }

    /// Try to get the raw ODBC Timestamp value
    pub fn timestamp(&self) -> Option<odbc_api::sys::Timestamp> {
        if self.is_null() {
            None
        } else {
            match &self.column_data.values {
                OdbcValueVec::Timestamp(raw_values) => raw_values.get(self.row_index).copied(),
                _ => None,
            }
        }
    }
}

/// Individual ODBC value type
#[derive(Debug, Clone)]
pub enum OdbcValueType {
    TinyInt(i8),
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Real(f32),
    Double(f64),
    Bit(odbc_api::Bit),
    Text(String),
    Binary(Vec<u8>),
    Date(odbc_api::sys::Date),
    Time(odbc_api::sys::Time),
    Timestamp(odbc_api::sys::Timestamp),
}

/// Convert AnySlice to owned OdbcValueVec and nulls vector, preserving original types
pub fn convert_any_slice_to_value_vec(slice: AnySlice<'_>) -> (OdbcValueVec, Vec<bool>) {
    match slice {
        AnySlice::I8(s) => (OdbcValueVec::TinyInt(s.to_vec()), vec![false; s.len()]),
        AnySlice::I16(s) => (OdbcValueVec::SmallInt(s.to_vec()), vec![false; s.len()]),
        AnySlice::I32(s) => (OdbcValueVec::Integer(s.to_vec()), vec![false; s.len()]),
        AnySlice::I64(s) => (OdbcValueVec::BigInt(s.to_vec()), vec![false; s.len()]),
        AnySlice::F32(s) => (OdbcValueVec::Real(s.to_vec()), vec![false; s.len()]),
        AnySlice::F64(s) => (OdbcValueVec::Double(s.to_vec()), vec![false; s.len()]),
        AnySlice::Bit(s) => (OdbcValueVec::Bit(s.to_vec()), vec![false; s.len()]),

        AnySlice::NullableI8(s) => {
            let values: Vec<Option<i8>> = s.map(|opt| opt.copied()).collect();
            let nulls = values.iter().map(|opt| opt.is_none()).collect();
            (
                OdbcValueVec::TinyInt(values.into_iter().map(|opt| opt.unwrap_or(0)).collect()),
                nulls,
            )
        }
        AnySlice::NullableI16(s) => {
            let values: Vec<Option<i16>> = s.map(|opt| opt.copied()).collect();
            let nulls = values.iter().map(|opt| opt.is_none()).collect();
            (
                OdbcValueVec::SmallInt(values.into_iter().map(|opt| opt.unwrap_or(0)).collect()),
                nulls,
            )
        }
        AnySlice::NullableI32(s) => {
            let values: Vec<Option<i32>> = s.map(|opt| opt.copied()).collect();
            let nulls = values.iter().map(|opt| opt.is_none()).collect();
            (
                OdbcValueVec::Integer(values.into_iter().map(|opt| opt.unwrap_or(0)).collect()),
                nulls,
            )
        }
        AnySlice::NullableI64(s) => {
            let values: Vec<Option<i64>> = s.map(|opt| opt.copied()).collect();
            let nulls = values.iter().map(|opt| opt.is_none()).collect();
            (
                OdbcValueVec::BigInt(values.into_iter().map(|opt| opt.unwrap_or(0)).collect()),
                nulls,
            )
        }
        AnySlice::NullableF32(s) => {
            let values: Vec<Option<f32>> = s.map(|opt| opt.copied()).collect();
            let nulls = values.iter().map(|opt| opt.is_none()).collect();
            (
                OdbcValueVec::Real(values.into_iter().map(|opt| opt.unwrap_or(0.0)).collect()),
                nulls,
            )
        }
        AnySlice::NullableF64(s) => {
            let values: Vec<Option<f64>> = s.map(|opt| opt.copied()).collect();
            let nulls = values.iter().map(|opt| opt.is_none()).collect();
            (
                OdbcValueVec::Double(values.into_iter().map(|opt| opt.unwrap_or(0.0)).collect()),
                nulls,
            )
        }
        AnySlice::NullableBit(s) => {
            let values: Vec<Option<odbc_api::Bit>> = s.map(|opt| opt.copied()).collect();
            let nulls = values.iter().map(|opt| opt.is_none()).collect();
            (
                OdbcValueVec::Bit(
                    values
                        .into_iter()
                        .map(|opt| opt.unwrap_or(odbc_api::Bit(0)))
                        .collect(),
                ),
                nulls,
            )
        }

        AnySlice::Text(s) => {
            let texts: Vec<Option<String>> = s
                .iter()
                .map(|bytes_opt| bytes_opt.map(|bytes| String::from_utf8_lossy(bytes).to_string()))
                .collect();
            let nulls = texts.iter().map(|opt| opt.is_none()).collect();
            (OdbcValueVec::Text(texts), nulls)
        }

        AnySlice::Binary(s) => {
            let binaries: Vec<Option<Vec<u8>>> = s
                .iter()
                .map(|bytes_opt| bytes_opt.map(|bytes| bytes.to_vec()))
                .collect();
            let nulls = binaries.iter().map(|opt| opt.is_none()).collect();
            (OdbcValueVec::Binary(binaries), nulls)
        }

        AnySlice::Date(s) => (OdbcValueVec::Date(s.to_vec()), vec![false; s.len()]),
        AnySlice::Time(s) => (OdbcValueVec::Time(s.to_vec()), vec![false; s.len()]),
        AnySlice::Timestamp(s) => (OdbcValueVec::Timestamp(s.to_vec()), vec![false; s.len()]),
        AnySlice::NullableDate(s) => {
            let (raw_values, indicators) = s.raw_values();
            let nulls = indicators.iter().map(|&ind| ind == NULL_DATA).collect();
            (OdbcValueVec::Date(raw_values.to_vec()), nulls)
        }
        AnySlice::NullableTime(s) => {
            let (raw_values, indicators) = s.raw_values();
            let nulls = indicators.iter().map(|&ind| ind == NULL_DATA).collect();
            (OdbcValueVec::Time(raw_values.to_vec()), nulls)
        }
        AnySlice::NullableTimestamp(s) => {
            let (raw_values, indicators) = s.raw_values();
            let nulls = indicators.iter().map(|&ind| ind == NULL_DATA).collect();
            (OdbcValueVec::Timestamp(raw_values.to_vec()), nulls)
        }

        _ => panic!("Unsupported AnySlice variant"),
    }
}

fn value_vec_is_null(column_data: &ColumnData, row_index: usize) -> bool {
    column_data.nulls.get(row_index).copied().unwrap_or(false)
}

fn value_vec_get_raw(values: &OdbcValueVec, row_index: usize) -> Option<OdbcValueType> {
    match values {
        OdbcValueVec::TinyInt(v) => v.get(row_index).map(|&val| OdbcValueType::TinyInt(val)),
        OdbcValueVec::SmallInt(v) => v.get(row_index).map(|&val| OdbcValueType::SmallInt(val)),
        OdbcValueVec::Integer(v) => v.get(row_index).map(|&val| OdbcValueType::Integer(val)),
        OdbcValueVec::BigInt(v) => v.get(row_index).map(|&val| OdbcValueType::BigInt(val)),
        OdbcValueVec::Real(v) => v.get(row_index).map(|&val| OdbcValueType::Real(val)),
        OdbcValueVec::Double(v) => v.get(row_index).map(|&val| OdbcValueType::Double(val)),
        OdbcValueVec::Bit(v) => v.get(row_index).map(|&val| OdbcValueType::Bit(val)),
        OdbcValueVec::Text(v) => v
            .get(row_index)
            .and_then(|opt| opt.clone().map(OdbcValueType::Text)),
        OdbcValueVec::Binary(v) => v
            .get(row_index)
            .and_then(|opt| opt.clone().map(OdbcValueType::Binary)),
        OdbcValueVec::Date(raw_values) => {
            raw_values.get(row_index).copied().map(OdbcValueType::Date)
        }
        OdbcValueVec::Time(raw_values) => {
            raw_values.get(row_index).copied().map(OdbcValueType::Time)
        }
        OdbcValueVec::Timestamp(raw_values) => raw_values
            .get(row_index)
            .copied()
            .map(OdbcValueType::Timestamp),
    }
}

pub trait TryFromInt:
    TryFrom<u8>
    + TryFrom<i16>
    + TryFrom<i32>
    + TryFrom<i64>
    + TryFrom<i8>
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
    + std::str::FromStr
{
}

impl<
        T: TryFrom<u8>
            + TryFrom<i16>
            + TryFrom<i32>
            + TryFrom<i64>
            + TryFrom<i8>
            + TryFrom<u16>
            + TryFrom<u32>
            + TryFrom<u64>
            + std::str::FromStr,
    > TryFromInt for T
{
}

fn value_vec_int<T: TryFromInt>(values: &OdbcValueVec, row_index: usize) -> Option<T> {
    match values {
        OdbcValueVec::TinyInt(v) => T::try_from(*v.get(row_index)?).ok(),
        OdbcValueVec::SmallInt(v) => T::try_from(*v.get(row_index)?).ok(),
        OdbcValueVec::Integer(v) => T::try_from(*v.get(row_index)?).ok(),
        OdbcValueVec::BigInt(v) => T::try_from(*v.get(row_index)?).ok(),
        OdbcValueVec::Bit(v) => T::try_from(v.get(row_index)?.0).ok(),
        OdbcValueVec::Text(v) => {
            if let Some(Some(text)) = v.get(row_index) {
                text.trim().parse().ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

pub trait TryFromFloat: TryFrom<f32> + TryFrom<f64> {}

impl<T: TryFrom<f32> + TryFrom<f64>> TryFromFloat for T {}

fn value_vec_float<T: TryFromFloat>(values: &OdbcValueVec, row_index: usize) -> Option<T> {
    match values {
        OdbcValueVec::Real(v) => T::try_from(*v.get(row_index)?).ok(),
        OdbcValueVec::Double(v) => T::try_from(*v.get(row_index)?).ok(),
        _ => None,
    }
}

fn value_vec_text(values: &OdbcValueVec, row_index: usize) -> Option<&str> {
    match values {
        OdbcValueVec::Text(v) => v.get(row_index).and_then(|opt| opt.as_deref()),
        _ => None,
    }
}

fn value_vec_blob(values: &OdbcValueVec, row_index: usize) -> Option<&[u8]> {
    match values {
        OdbcValueVec::Binary(v) => v.get(row_index).and_then(|opt| opt.as_deref()),
        _ => None,
    }
}

// Decode implementations have been moved to the types module

#[cfg(feature = "any")]
impl<'r> From<OdbcValueRef<'r>> for crate::any::AnyValueRef<'r> {
    fn from(value: OdbcValueRef<'r>) -> Self {
        crate::any::AnyValueRef {
            type_info: crate::any::AnyTypeInfo::from(value.column_data.type_info.clone()),
            kind: crate::any::value::AnyValueRefKind::Odbc(value),
        }
    }
}

#[cfg(feature = "any")]
impl From<OdbcValue> for crate::any::AnyValue {
    fn from(value: OdbcValue) -> Self {
        crate::any::AnyValue {
            type_info: crate::any::AnyTypeInfo::from(value.column_data.type_info.clone()),
            kind: crate::any::value::AnyValueKind::Odbc(value),
        }
    }
}
