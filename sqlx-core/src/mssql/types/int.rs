use std::any::type_name;
use std::convert::TryFrom;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for i8 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 1))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::TinyInt | DataType::IntN) && ty.0.size == 1
    }
}

impl Encode<'_, Mssql> for i8 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

fn decode_int_bytes<T, U, const N: usize>(
    bytes: &[u8],
    type_info: &MssqlTypeInfo,
    from_le_bytes: impl Fn([u8; N]) -> U,
) -> Result<T, BoxDynError>
where
    T: TryFrom<U>,
    T::Error: std::error::Error + Send + Sync + 'static,
    U: std::fmt::Display + Copy,
{
    if bytes.len() != N {
        return Err(err_protocol!(
            "{} should have exactly {} byte(s), got {}",
            type_info,
            N,
            bytes.len()
        )
        .into());
    }

    let mut buf = [0u8; N];
    buf.copy_from_slice(bytes);
    let val = from_le_bytes(buf);

    T::try_from(val).map_err(|err| {
        err_protocol!(
            "Converting {} {} to {} failed: {}",
            type_info,
            val,
            type_name::<T>(),
            err
        )
        .into()
    })
}

fn decode_tinyint<T>(bytes: &[u8], type_info: &MssqlTypeInfo) -> Result<T, BoxDynError>
where
    T: TryFrom<u8>,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    if bytes.len() != 1 {
        return Err(err_protocol!(
            "{} should have exactly 1 byte, got {}",
            type_info,
            bytes.len()
        )
        .into());
    }

    let val = u8::from_le_bytes([bytes[0]]);
    T::try_from(val).map_err(|err| {
        err_protocol!(
            "Converting {} {} to {} failed: {}",
            type_info,
            val,
            type_name::<T>(),
            err
        )
        .into()
    })
}

fn decode_int_direct<T>(value: MssqlValueRef<'_>) -> Result<T, BoxDynError>
where
    T: TryFrom<i64> + TryFrom<u8> + TryFrom<i16> + TryFrom<i32>,
    <T as TryFrom<i64>>::Error: std::error::Error + Send + Sync + 'static,
    <T as TryFrom<u8>>::Error: std::error::Error + Send + Sync + 'static,
    <T as TryFrom<i16>>::Error: std::error::Error + Send + Sync + 'static,
    <T as TryFrom<i32>>::Error: std::error::Error + Send + Sync + 'static,
{
    let type_info = &value.type_info;
    let ty = type_info.0.ty;
    let precision = type_info.0.precision;
    let scale = type_info.0.scale;
    let bytes_val = value.as_bytes()?;

    match ty {
        DataType::TinyInt => decode_tinyint(bytes_val, type_info),
        DataType::SmallInt => decode_int_bytes(bytes_val, type_info, i16::from_le_bytes),
        DataType::Int => decode_int_bytes(bytes_val, type_info, i32::from_le_bytes),
        DataType::BigInt => decode_int_bytes(bytes_val, type_info, i64::from_le_bytes),
        DataType::IntN => match bytes_val.len() {
            1 => decode_tinyint(bytes_val, type_info),
            2 => decode_int_bytes(bytes_val, type_info, i16::from_le_bytes),
            4 => decode_int_bytes(bytes_val, type_info, i32::from_le_bytes),
            8 => decode_int_bytes(bytes_val, type_info, i64::from_le_bytes),
            len => Err(err_protocol!("IntN with {} bytes is not supported", len).into()),
        },
        DataType::Numeric | DataType::NumericN | DataType::Decimal | DataType::DecimalN => {
            let i64_val = decode_numeric(bytes_val, precision, scale)?;
            convert_integer::<T>(i64_val)
        }
        _ => Err(err_protocol!(
            "Decoding {:?} as {} failed because type {:?} is not supported",
            value,
            type_name::<T>(),
            ty
        )
        .into()),
    }
}

impl Decode<'_, Mssql> for i8 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_int_direct(value)
    }
}

impl Type<Mssql> for i16 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 2))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::TinyInt | DataType::SmallInt | DataType::Int | DataType::IntN
        ) && ty.0.size <= 2
    }
}

impl Encode<'_, Mssql> for i16 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for i16 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_int_direct(value)
    }
}

impl Type<Mssql> for i32 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 4))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Int | DataType::IntN) && ty.0.size == 4
    }
}

impl Encode<'_, Mssql> for i32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for i32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_int_direct(value)
    }
}

impl Type<Mssql> for i64 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 8))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::SmallInt
                | DataType::Int
                | DataType::TinyInt
                | DataType::BigInt
                | DataType::IntN
                | DataType::Numeric
                | DataType::NumericN
                | DataType::Decimal
                | DataType::DecimalN
        )
    }
}

impl Encode<'_, Mssql> for i64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for i64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_int_direct(value)
    }
}

fn decode_numeric(bytes: &[u8], _precision: u8, mut scale: u8) -> Result<i64, BoxDynError> {
    let negative = bytes[0] == 0;
    let rest = &bytes[1..];
    let mut fixed_bytes = [0u8; 16];
    fixed_bytes[0..rest.len()].copy_from_slice(rest);
    let mut numerator = u128::from_le_bytes(fixed_bytes);
    while numerator % 10 == 0 && scale > 0 {
        numerator /= 10;
        scale -= 1;
    }
    if scale > 0 {
        numerator /= 10u128.pow(scale as u32);
    }
    let n = i64::try_from(numerator)?;
    Ok(n * if negative { -1 } else { 1 })
}

fn convert_integer<T>(i64_val: i64) -> Result<T, BoxDynError>
where
    T: TryFrom<i64>,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    T::try_from(i64_val).map_err(|err| {
        err_protocol!(
            "Converting {} to {} failed: {}",
            i64_val,
            type_name::<T>(),
            err
        )
        .into()
    })
}
