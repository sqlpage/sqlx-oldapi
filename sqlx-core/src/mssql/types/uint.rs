use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for u8 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 1))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::TinyInt | DataType::IntN) && ty.0.size == 1
    }
}

impl Encode<'_, Mssql> for u8 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&[*self]);
        IsNull::No
    }
}

impl Decode<'_, Mssql> for u8 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(*value
            .as_bytes()?
            .first()
            .ok_or("Invalid numeric value length")?)
    }
}

impl Type<Mssql> for u16 {
    fn type_info() -> MssqlTypeInfo {
        <i16 as Type<Mssql>>::type_info()
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <i16 as Type<Mssql>>::compatible(ty)
    }
}

impl Encode<'_, Mssql> for u16 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let v = *self as i16;
        <i16 as Encode<'_, Mssql>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Mssql> for u16 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let i64_val = <i64 as Decode<Mssql>>::decode(value)?;
        super::int::convert_integer::<Self>(i64_val)
    }
}

impl Type<Mssql> for u32 {
    fn type_info() -> MssqlTypeInfo {
        <i32 as Type<Mssql>>::type_info()
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <i32 as Type<Mssql>>::compatible(ty)
    }
}

impl Encode<'_, Mssql> for u32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let v = *self as i32;
        <i32 as Encode<'_, Mssql>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Mssql> for u32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let i64_val = <i64 as Decode<Mssql>>::decode(value)?;
        super::int::convert_integer::<Self>(i64_val)
    }
}

impl Type<Mssql> for u64 {
    fn type_info() -> MssqlTypeInfo {
        <i64 as Type<Mssql>>::type_info()
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <i64 as Type<Mssql>>::compatible(ty)
    }
}

impl Encode<'_, Mssql> for u64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let v = *self as i64;
        <i64 as Encode<'_, Mssql>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Mssql> for u64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = <i64 as Decode<'_, Mssql>>::decode(value)?;
        Ok(u64::try_from(v)?)
    }
}
