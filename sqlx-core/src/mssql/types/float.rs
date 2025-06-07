use byteorder::{ByteOrder, LittleEndian};

use super::decimal_tools::{decode_money_bytes, decode_numeric_bytes};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for f32 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::FloatN, 4))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <f64 as Type<Mssql>>::compatible(ty)
    }
}

impl Encode<'_, Mssql> for f32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for f32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let as_f64 = <f64 as Decode<'_, Mssql>>::decode(value)?;
        Ok(as_f64 as f32)
    }
}

impl Type<Mssql> for f64 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::FloatN, 8))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::Float
                | DataType::FloatN
                | DataType::Decimal
                | DataType::DecimalN
                | DataType::Numeric
                | DataType::NumericN
                | DataType::MoneyN
                | DataType::Money
                | DataType::SmallMoney
        )
    }
}

impl Encode<'_, Mssql> for f64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for f64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.0.ty;
        let size = value.type_info.0.size;
        let precision = value.type_info.0.precision;
        let scale = value.type_info.0.scale;
        match ty {
            DataType::Float | DataType::FloatN if size == 8 => {
                Ok(LittleEndian::read_f64(value.as_bytes()?))
            }
            DataType::Float | DataType::FloatN if size == 4 => {
                Ok(f64::from(LittleEndian::read_f32(value.as_bytes()?)))
            }
            DataType::Numeric | DataType::NumericN | DataType::Decimal | DataType::DecimalN => {
                decode_numeric(value.as_bytes()?, precision, scale)
            }
            DataType::MoneyN | DataType::Money | DataType::SmallMoney => {
                let numerator = decode_money_bytes(value.as_bytes()?)?;
                let denominator = 10_000;
                let integer_part = (numerator / denominator) as f64;
                let fractional_part = (numerator % denominator) as f64 / denominator as f64;
                Ok(integer_part + fractional_part)
            }
            _ => Err(err_protocol!(
                "Decoding {:?} as a float failed because type {:?} is not implemented",
                value,
                ty
            )
            .into()),
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn decode_numeric(bytes: &[u8], _precision: u8, mut scale: u8) -> Result<f64, BoxDynError> {
    let (sign, mut numerator) = decode_numeric_bytes(bytes)?;

    while numerator % 10 == 0 && scale > 0 {
        numerator /= 10;
        scale -= 1;
    }
    let denominator = 10u128.pow(scale as u32);
    let integer_part = (numerator / denominator) as f64;
    let fractional_part = (numerator % denominator) as f64 / denominator as f64;
    let absolute = integer_part + fractional_part;
    let positive = sign == 1;
    Ok(if positive { absolute } else { -absolute })
}
