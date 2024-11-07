use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for u8 {
    fn type_info() -> MssqlTypeInfo {
        <i8 as Type<Mssql>>::type_info()
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <i8 as Type<Mssql>>::compatible(ty)
    }
}

impl Encode<'_, Mssql> for u8 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let v = i8::try_from(*self).unwrap_or_else(|_e| {
            log::warn!("cannot encode {self} as a signed mssql tinyint");
            i8::MAX
        });
        <i8 as Encode<'_, Mssql>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Mssql> for u8 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = <i8 as Decode<'_, Mssql>>::decode(value)?;
        Ok(u8::try_from(v)?)
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
        let v = i16::try_from(*self).unwrap_or_else(|_e| {
            log::warn!("cannot encode {self} as a signed mssql smallint");
            i16::MAX
        });
        <i16 as Encode<'_, Mssql>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Mssql> for u16 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = <i16 as Decode<'_, Mssql>>::decode(value)?;
        Ok(u16::try_from(v)?)
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
        let v = i32::try_from(*self).unwrap_or_else(|_e| {
            log::warn!("cannot encode {self} as a signed mssql int");
            i32::MAX
        });
        <i32 as Encode<'_, Mssql>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Mssql> for u32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = <i32 as Decode<'_, Mssql>>::decode(value)?;
        Ok(u32::try_from(v)?)
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
        let v = i64::try_from(*self).unwrap_or_else(|_e| {
            log::warn!("cannot encode {self} as a signed mssql bigint");
            i64::MAX
        });
        <i64 as Encode<'_, Mssql>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Mssql> for u64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = <i64 as Decode<'_, Mssql>>::decode(value)?;
        Ok(u64::try_from(v)?)
    }
}
