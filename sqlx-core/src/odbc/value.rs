use crate::error::Error;
use crate::odbc::{Odbc, OdbcBatch, OdbcTypeInfo};
use crate::type_info::TypeInfo;
use crate::value::{Value, ValueRef};
use odbc_api::buffers::{AnySlice, NullableSlice};
use odbc_api::handles::CDataMut;
use odbc_api::parameter::CElement;
use odbc_api::sys::NULL_DATA;
use odbc_api::{DataType, Nullable};
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
    Bit(Vec<bool>),

    // Text types (inherently nullable in ODBC)
    Text(Vec<String>),

    // Binary types (inherently nullable in ODBC)
    Binary(Vec<Vec<u8>>),

    // Date/Time types
    Date(Vec<odbc_api::sys::Date>),
    Time(Vec<odbc_api::sys::Time>),
    Timestamp(Vec<odbc_api::sys::Timestamp>),
}

impl OdbcValueVec {
    pub(crate) fn with_capacity_for_type(data_type: DataType, capacity: usize) -> Self {
        match data_type {
            DataType::TinyInt => OdbcValueVec::TinyInt(Vec::with_capacity(capacity)),
            DataType::SmallInt => OdbcValueVec::SmallInt(Vec::with_capacity(capacity)),
            // Some drivers report INTEGER even when the value range is 64-bit.
            DataType::Integer | DataType::BigInt => {
                OdbcValueVec::BigInt(Vec::with_capacity(capacity))
            }
            DataType::Real => OdbcValueVec::Real(Vec::with_capacity(capacity)),
            DataType::Float { .. } | DataType::Double => {
                OdbcValueVec::Double(Vec::with_capacity(capacity))
            }
            DataType::Bit => OdbcValueVec::Bit(Vec::with_capacity(capacity)),
            DataType::Date => OdbcValueVec::Date(Vec::with_capacity(capacity)),
            DataType::Time { .. } => OdbcValueVec::Time(Vec::with_capacity(capacity)),
            DataType::Timestamp { .. } => OdbcValueVec::Timestamp(Vec::with_capacity(capacity)),
            DataType::Binary { .. }
            | DataType::Varbinary { .. }
            | DataType::LongVarbinary { .. } => OdbcValueVec::Binary(Vec::with_capacity(capacity)),
            _ => OdbcValueVec::Text(Vec::with_capacity(capacity)),
        }
    }

    pub(crate) fn push_from_cursor_row(
        &mut self,
        cursor_row: &mut odbc_api::CursorRow<'_>,
        col_index: u16,
        nulls: &mut Vec<bool>,
    ) -> Result<(), odbc_api::Error> {
        match self {
            OdbcValueVec::TinyInt(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::SmallInt(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Integer(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::BigInt(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Real(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Double(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Bit(v) => push_bit(cursor_row, col_index, v, nulls),
            OdbcValueVec::Date(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Time(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Timestamp(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Binary(v) => push_binary(cursor_row, col_index, v, nulls),
            OdbcValueVec::Text(v) => push_text(cursor_row, col_index, v, nulls),
        }
    }
}

fn push_get_data<T: Default + Copy + CElement + CDataMut>(
    cursor_row: &mut odbc_api::CursorRow<'_>,
    col_index: u16,
    vec: &mut Vec<T>,
    nulls: &mut Vec<bool>,
) -> Result<(), odbc_api::Error>
where
    Nullable<T>: CElement + CDataMut,
{
    let mut tmp = Nullable::null();
    cursor_row.get_data(col_index, &mut tmp)?;
    let option = tmp.into_opt();
    nulls.push(option.is_none());
    vec.push(option.unwrap_or_default());
    Ok(())
}

fn push_binary(
    cursor_row: &mut odbc_api::CursorRow<'_>,
    col_index: u16,
    vec: &mut Vec<Vec<u8>>,
    nulls: &mut Vec<bool>,
) -> Result<(), odbc_api::Error> {
    let mut buf = Vec::new();
    let is_not_null = cursor_row.get_binary(col_index, &mut buf)?;
    nulls.push(!is_not_null);
    vec.push(buf);
    Ok(())
}

fn push_text(
    cursor_row: &mut odbc_api::CursorRow<'_>,
    col_index: u16,
    vec: &mut Vec<String>,
    nulls: &mut Vec<bool>,
) -> Result<(), odbc_api::Error> {
    let mut buf = Vec::<u16>::new();
    let is_not_null = cursor_row.get_wide_text(col_index, &mut buf)?;
    vec.push(String::from_utf16_lossy(&buf).to_string());
    nulls.push(!is_not_null);
    Ok(())
}

fn push_bit(
    cursor_row: &mut odbc_api::CursorRow<'_>,
    col_index: u16,
    vec: &mut Vec<bool>,
    nulls: &mut Vec<bool>,
) -> Result<(), odbc_api::Error> {
    let mut bit_val = Nullable::<odbc_api::Bit>::null();
    cursor_row.get_data(col_index, &mut bit_val)?;
    match bit_val.into_opt() {
        Some(bit) => {
            nulls.push(false);
            vec.push(bit.as_bool());
        }
        None => {
            nulls.push(true);
            vec.push(false);
        }
    }
    Ok(())
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
    pub(crate) batch: &'r OdbcBatch,
    pub(crate) row_index: usize,
    pub(crate) column_index: usize,
}

#[derive(Debug, Clone)]
pub struct OdbcValue {
    pub(crate) batch: Arc<OdbcBatch>,
    pub(crate) row_index: usize,
    pub(crate) column_index: usize,
}

impl<'r> ValueRef<'r> for OdbcValueRef<'r> {
    type Database = Odbc;

    fn to_owned(&self) -> OdbcValue {
        OdbcValue {
            batch: Arc::new(self.batch.clone()),
            row_index: self.row_index,
            column_index: self.column_index,
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.batch.column_data[self.column_index].type_info)
    }

    fn is_null(&self) -> bool {
        value_vec_is_null(&self.batch.column_data[self.column_index], self.row_index)
    }
}

impl Value for OdbcValue {
    type Database = Odbc;

    fn as_ref(&self) -> OdbcValueRef<'_> {
        OdbcValueRef {
            batch: &self.batch,
            row_index: self.row_index,
            column_index: self.column_index,
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.batch.column_data[self.column_index].type_info)
    }

    fn is_null(&self) -> bool {
        value_vec_is_null(&self.batch.column_data[self.column_index], self.row_index)
    }
}

/// Utility methods for OdbcValue
impl OdbcValue {
    /// Create a new OdbcValue from batch, row index, and column index
    pub fn new(batch: Arc<OdbcBatch>, row_index: usize, column_index: usize) -> Self {
        Self {
            batch,
            row_index,
            column_index,
        }
    }

    /// Get the raw value from the column data
    pub fn get_raw(&self) -> Option<OdbcValueType> {
        value_vec_get_raw(&self.batch.column_data[self.column_index], self.row_index)
    }

    /// Try to get the value as i64
    pub fn as_int<T: TryFromInt>(&self) -> Option<T> {
        value_vec_int(&self.batch.column_data[self.column_index], self.row_index)
    }

    /// Try to get the value as f64
    pub fn as_f64(&self) -> Option<f64> {
        value_vec_float(&self.batch.column_data[self.column_index], self.row_index)
    }

    /// Try to get the value as string
    pub fn as_str(&self) -> Option<Cow<'_, str>> {
        value_vec_text(&self.batch.column_data[self.column_index], self.row_index)
            .map(Cow::Borrowed)
    }

    /// Try to get the value as bytes
    pub fn as_bytes(&self) -> Option<Cow<'_, [u8]>> {
        value_vec_blob(&self.batch.column_data[self.column_index], self.row_index)
            .map(Cow::Borrowed)
    }
}

/// Utility methods for OdbcValueRef
impl<'r> OdbcValueRef<'r> {
    /// Create a new OdbcValueRef from batch, row index, and column index
    pub fn new(batch: &'r OdbcBatch, row_index: usize, column_index: usize) -> Self {
        Self {
            batch,
            row_index,
            column_index,
        }
    }

    /// Get the raw value from the column data
    pub fn get_raw(&self) -> Option<OdbcValueType> {
        value_vec_get_raw(&self.batch.column_data[self.column_index], self.row_index)
    }

    /// Try to get the value as i64
    pub fn int<T: TryFromInt>(&self) -> Option<T> {
        value_vec_int(&self.batch.column_data[self.column_index], self.row_index)
    }

    pub fn try_int<T: TryFromInt + crate::types::Type<Odbc>>(&self) -> crate::error::Result<T> {
        self.int::<T>().ok_or_else(|| {
            crate::error::Error::Decode(Box::new(crate::error::MismatchedTypeError {
                rust_type: std::any::type_name::<T>().to_string(),
                rust_sql_type: T::type_info().name().to_string(),
                sql_type: self.batch.column_data[self.column_index]
                    .type_info
                    .name()
                    .to_string(),
                source: Some(format!("ODBC: cannot decode {:?}", self).into()),
            }))
        })
    }

    pub fn try_float<T: TryFromFloat + crate::types::Type<Odbc>>(&self) -> crate::error::Result<T> {
        self.float::<T>().ok_or_else(|| {
            crate::error::Error::Decode(Box::new(crate::error::MismatchedTypeError {
                rust_type: std::any::type_name::<T>().to_string(),
                rust_sql_type: T::type_info().name().to_string(),
                sql_type: self.batch.column_data[self.column_index]
                    .type_info
                    .name()
                    .to_string(),
                source: Some(format!("ODBC: cannot decode {:?}", self).into()),
            }))
        })
    }

    /// Try to get the value as f64
    pub fn float<T: TryFromFloat>(&self) -> Option<T> {
        value_vec_float(&self.batch.column_data[self.column_index], self.row_index)
    }

    /// Try to get the value as string slice
    pub fn text(&self) -> Option<&'r str> {
        value_vec_text(&self.batch.column_data[self.column_index], self.row_index)
    }

    /// Try to get the value as binary slice
    pub fn blob(&self) -> Option<&'r [u8]> {
        value_vec_blob(&self.batch.column_data[self.column_index], self.row_index)
    }

    /// Try to get the raw ODBC Date value
    pub fn date(&self) -> Option<odbc_api::sys::Date> {
        if self.is_null() {
            None
        } else {
            match &self.batch.column_data[self.column_index].values {
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
            match &self.batch.column_data[self.column_index].values {
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
            match &self.batch.column_data[self.column_index].values {
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
    Bit(bool),
    Text(String),
    Binary(Vec<u8>),
    Date(odbc_api::sys::Date),
    Time(odbc_api::sys::Time),
    Timestamp(odbc_api::sys::Timestamp),
}

/// Generic helper function to handle non-nullable slices
fn handle_non_nullable_slice<T: Copy>(
    slice: &[T],
    constructor: fn(Vec<T>) -> OdbcValueVec,
) -> (OdbcValueVec, Vec<bool>) {
    let vec = slice.to_vec();
    (constructor(vec), vec![false; slice.len()])
}

/// Generic helper function to handle nullable slices with custom default values
fn handle_nullable_slice<'a, T: Default + Copy>(
    slice: NullableSlice<'a, T>,
    constructor: fn(Vec<T>) -> OdbcValueVec,
) -> (OdbcValueVec, Vec<bool>) {
    let size = slice.size_hint().1.unwrap_or(0);
    let mut values = Vec::with_capacity(size);
    let mut nulls = Vec::with_capacity(size);
    for opt in slice {
        values.push(opt.copied().unwrap_or_default());
        nulls.push(opt.is_none());
    }
    (constructor(values), nulls)
}

/// Generic helper function to handle nullable slices with NULL_DATA indicators
fn handle_nullable_with_indicators<T: Default + Copy>(
    raw_values: &[T],
    indicators: &[isize],
    constructor: fn(Vec<T>) -> OdbcValueVec,
) -> (OdbcValueVec, Vec<bool>) {
    let nulls = indicators.iter().map(|&ind| ind == NULL_DATA).collect();
    (constructor(raw_values.to_vec()), nulls)
}

fn handle_non_nullable_u8_slice(slice: &[u8]) -> (OdbcValueVec, Vec<bool>) {
    (
        OdbcValueVec::BigInt(slice.iter().map(|&value| i64::from(value)).collect()),
        vec![false; slice.len()],
    )
}

fn handle_nullable_u8_slice(slice: NullableSlice<'_, u8>) -> (OdbcValueVec, Vec<bool>) {
    let size = slice.size_hint().1.unwrap_or(0);
    let mut values = Vec::with_capacity(size);
    let mut nulls = Vec::with_capacity(size);

    for opt in slice {
        values.push(opt.copied().map(i64::from).unwrap_or_default());
        nulls.push(opt.is_none());
    }

    (OdbcValueVec::BigInt(values), nulls)
}

/// Convert AnySlice to owned OdbcValueVec and nulls vector, preserving original types
pub(crate) fn convert_any_slice_to_value_vec(
    slice: AnySlice<'_>,
) -> Result<(OdbcValueVec, Vec<bool>), Error> {
    Ok(match slice {
        // Non-nullable integer types
        AnySlice::I8(s) => handle_non_nullable_slice(s, OdbcValueVec::TinyInt),
        AnySlice::I16(s) => handle_non_nullable_slice(s, OdbcValueVec::SmallInt),
        AnySlice::I32(s) => handle_non_nullable_slice(s, OdbcValueVec::Integer),
        AnySlice::I64(s) => handle_non_nullable_slice(s, OdbcValueVec::BigInt),
        AnySlice::U8(s) => handle_non_nullable_u8_slice(s),

        // Non-nullable floating point types
        AnySlice::F32(s) => handle_non_nullable_slice(s, OdbcValueVec::Real),
        AnySlice::F64(s) => handle_non_nullable_slice(s, OdbcValueVec::Double),

        // Non-nullable other types
        AnySlice::Bit(s) => {
            let vec: Vec<bool> = s.iter().map(|bit| bit.as_bool()).collect();
            (OdbcValueVec::Bit(vec), vec![false; s.len()])
        }
        AnySlice::Date(s) => handle_non_nullable_slice(s, OdbcValueVec::Date),
        AnySlice::Time(s) => handle_non_nullable_slice(s, OdbcValueVec::Time),
        AnySlice::Timestamp(s) => handle_non_nullable_slice(s, OdbcValueVec::Timestamp),

        // Nullable integer types
        AnySlice::NullableI8(s) => handle_nullable_slice(s, OdbcValueVec::TinyInt),
        AnySlice::NullableI16(s) => handle_nullable_slice(s, OdbcValueVec::SmallInt),
        AnySlice::NullableI32(s) => handle_nullable_slice(s, OdbcValueVec::Integer),
        AnySlice::NullableI64(s) => handle_nullable_slice(s, OdbcValueVec::BigInt),
        AnySlice::NullableU8(s) => handle_nullable_u8_slice(s),
        AnySlice::NullableF32(s) => handle_nullable_slice(s, OdbcValueVec::Real),
        AnySlice::NullableF64(s) => handle_nullable_slice(s, OdbcValueVec::Double),
        AnySlice::NullableBit(s) => {
            let values: Vec<Option<odbc_api::Bit>> = s.map(|opt| opt.copied()).collect();
            let nulls = values.iter().map(|opt| opt.is_none()).collect();
            (
                OdbcValueVec::Bit(
                    values
                        .into_iter()
                        .map(|opt| opt.is_some_and(|bit| bit.as_bool()))
                        .collect(),
                ),
                nulls,
            )
        }

        // Text and binary types (inherently nullable)
        AnySlice::Text(s) => {
            let mut values = Vec::with_capacity(s.len());
            let mut nulls = Vec::with_capacity(s.len());
            for bytes_opt in s.iter() {
                nulls.push(bytes_opt.is_none());
                values.push(String::from_utf8_lossy(bytes_opt.unwrap_or_default()).into_owned());
            }
            (OdbcValueVec::Text(values), nulls)
        }
        AnySlice::WText(s) => {
            let mut values = Vec::with_capacity(s.len());
            let mut nulls = Vec::with_capacity(s.len());
            for chars_opt in s.iter() {
                nulls.push(chars_opt.is_none());
                values.push(
                    chars_opt
                        .map(|chars| String::from_utf16_lossy(chars.into()))
                        .unwrap_or_default(),
                );
            }
            (OdbcValueVec::Text(values), nulls)
        }
        AnySlice::Binary(s) => {
            let mut values = Vec::with_capacity(s.len());
            let mut nulls = Vec::with_capacity(s.len());
            for bytes_opt in s.iter() {
                nulls.push(bytes_opt.is_none());
                values.push(bytes_opt.unwrap_or_default().to_vec());
            }
            (OdbcValueVec::Binary(values), nulls)
        }

        // Nullable date/time types with NULL_DATA indicators
        AnySlice::NullableDate(s) => {
            let (raw_values, indicators) = s.raw_values();
            handle_nullable_with_indicators(raw_values, indicators, OdbcValueVec::Date)
        }
        AnySlice::NullableTime(s) => {
            let (raw_values, indicators) = s.raw_values();
            handle_nullable_with_indicators(raw_values, indicators, OdbcValueVec::Time)
        }
        AnySlice::NullableTimestamp(s) => {
            let (raw_values, indicators) = s.raw_values();
            handle_nullable_with_indicators(raw_values, indicators, OdbcValueVec::Timestamp)
        }

        unsupported => {
            return Err(Error::Protocol(format!(
                "unsupported ODBC buffer slice variant: {:?}",
                std::mem::discriminant(&unsupported)
            )));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_unsigned_tinyint_slices() {
        let (values, nulls) = convert_any_slice_to_value_vec(AnySlice::U8(&[0, 255])).unwrap();

        assert_eq!(nulls, vec![false, false]);
        assert!(matches!(values, OdbcValueVec::BigInt(values) if values == vec![0, 255]));
    }
}

fn value_vec_is_null(column_data: &ColumnData, row_index: usize) -> bool {
    column_data.nulls.get(row_index).copied().unwrap_or(false)
}

macro_rules! impl_get_raw_arm_copy {
    ($vec:expr, $row_index:expr, $variant:ident, $type:ty) => {
        $vec.get($row_index).copied().map(OdbcValueType::$variant)
    };
}

fn value_vec_get_raw(column_data: &ColumnData, row_index: usize) -> Option<OdbcValueType> {
    if value_vec_is_null(column_data, row_index) {
        return None;
    }
    match &column_data.values {
        OdbcValueVec::TinyInt(v) => v.get(row_index).map(|&val| OdbcValueType::TinyInt(val)),
        OdbcValueVec::SmallInt(v) => v.get(row_index).map(|&val| OdbcValueType::SmallInt(val)),
        OdbcValueVec::Integer(v) => v.get(row_index).map(|&val| OdbcValueType::Integer(val)),
        OdbcValueVec::BigInt(v) => v.get(row_index).map(|&val| OdbcValueType::BigInt(val)),
        OdbcValueVec::Real(v) => v.get(row_index).map(|&val| OdbcValueType::Real(val)),
        OdbcValueVec::Double(v) => v.get(row_index).map(|&val| OdbcValueType::Double(val)),
        OdbcValueVec::Bit(v) => v.get(row_index).map(|&val| OdbcValueType::Bit(val)),
        OdbcValueVec::Text(v) => v.get(row_index).cloned().map(OdbcValueType::Text),
        OdbcValueVec::Binary(v) => v.get(row_index).cloned().map(OdbcValueType::Binary),
        OdbcValueVec::Date(v) => impl_get_raw_arm_copy!(v, row_index, Date, odbc_api::sys::Date),
        OdbcValueVec::Time(v) => impl_get_raw_arm_copy!(v, row_index, Time, odbc_api::sys::Time),
        OdbcValueVec::Timestamp(v) => {
            impl_get_raw_arm_copy!(v, row_index, Timestamp, odbc_api::sys::Timestamp)
        }
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

macro_rules! impl_int_conversion {
    ($vec:expr, $row_index:expr, $type:ty) => {
        <$type>::try_from(*$vec.get($row_index)?).ok()
    };
    ($vec:expr, $row_index:expr, $type:ty, text) => {
        if let Some(Some(text)) = $vec.get($row_index) {
            text.trim().parse().ok()
        } else {
            None
        }
    };
}

fn value_vec_int<T: TryFromInt>(column_data: &ColumnData, row_index: usize) -> Option<T> {
    if value_vec_is_null(column_data, row_index) {
        return None;
    }
    match &column_data.values {
        OdbcValueVec::TinyInt(v) => impl_int_conversion!(v, row_index, T),
        OdbcValueVec::SmallInt(v) => impl_int_conversion!(v, row_index, T),
        OdbcValueVec::Integer(v) => impl_int_conversion!(v, row_index, T),
        OdbcValueVec::BigInt(v) => impl_int_conversion!(v, row_index, T),
        OdbcValueVec::Bit(v) => T::try_from(*v.get(row_index)? as u8).ok(),
        OdbcValueVec::Text(v) => v.get(row_index).and_then(|text| text.trim().parse().ok()),
        _ => None,
    }
}

pub trait TryFromFloat: TryFrom<f32> + TryFrom<f64> {}

impl<T: TryFrom<f32> + TryFrom<f64>> TryFromFloat for T {}

macro_rules! impl_float_conversion {
    ($vec:expr, $row_index:expr, $type:ty) => {
        <$type>::try_from(*$vec.get($row_index)?).ok()
    };
}

fn value_vec_float<T: TryFromFloat>(column_data: &ColumnData, row_index: usize) -> Option<T> {
    if value_vec_is_null(column_data, row_index) {
        return None;
    }
    match &column_data.values {
        OdbcValueVec::Real(v) => impl_float_conversion!(v, row_index, T),
        OdbcValueVec::Double(v) => impl_float_conversion!(v, row_index, T),
        _ => None,
    }
}

fn value_vec_text(column_data: &ColumnData, row_index: usize) -> Option<&str> {
    if value_vec_is_null(column_data, row_index) {
        return None;
    }
    match &column_data.values {
        OdbcValueVec::Text(v) => v.get(row_index).map(|s| s.as_str()),
        _ => None,
    }
}

fn value_vec_blob(column_data: &ColumnData, row_index: usize) -> Option<&[u8]> {
    if value_vec_is_null(column_data, row_index) {
        return None;
    }
    match &column_data.values {
        OdbcValueVec::Binary(v) => v.get(row_index).map(|b| b.as_slice()),
        _ => None,
    }
}

#[cfg(feature = "any")]
impl<'r> From<OdbcValueRef<'r>> for crate::any::AnyValueRef<'r> {
    fn from(value: OdbcValueRef<'r>) -> Self {
        crate::any::AnyValueRef {
            type_info: crate::any::AnyTypeInfo::from(
                value.batch.column_data[value.column_index]
                    .type_info
                    .clone(),
            ),
            kind: crate::any::value::AnyValueRefKind::Odbc(value),
        }
    }
}

#[cfg(feature = "any")]
impl From<OdbcValue> for crate::any::AnyValue {
    fn from(value: OdbcValue) -> Self {
        crate::any::AnyValue {
            type_info: crate::any::AnyTypeInfo::from(
                value.batch.column_data[value.column_index]
                    .type_info
                    .clone(),
            ),
            kind: crate::any::value::AnyValueKind::Odbc(value),
        }
    }
}
