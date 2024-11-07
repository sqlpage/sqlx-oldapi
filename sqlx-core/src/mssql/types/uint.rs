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
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, Mssql> for u8 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value.as_bytes()?[0])
    }
}

impl Type<Mssql> for u16 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 4))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Int | DataType::IntN) && ty.0.size == 4
    }
}

impl Encode<'_, Mssql> for u16 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let value = i32::from(*self);
        buf.extend(&value.to_le_bytes());
        IsNull::No
    }
}

impl Decode<'_, Mssql> for u16 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        let val = i32::from_le_bytes(bytes.try_into()?);
        u16::try_from(val).map_err(Into::into)
    }
}

impl Type<Mssql> for u32 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::IntN, 8))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::BigInt | DataType::IntN) && ty.0.size == 8
    }
}

impl Encode<'_, Mssql> for u32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let value = i64::from(*self);
        buf.extend(&value.to_le_bytes());
        IsNull::No
    }
}

impl Decode<'_, Mssql> for u32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        let val = i64::from_le_bytes(bytes.try_into()?);
        u32::try_from(val).map_err(Into::into)
    }
}

impl Type<Mssql> for u64 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::NumericN, 17))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::Numeric
                | DataType::NumericN
                | DataType::Decimal
                | DataType::DecimalN
        ) && (ty.0.size == 0 || ty.0.size == 17)
    }
}

impl Encode<'_, Mssql> for u64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let bytes = self.to_le_bytes();
        let scale = 0i8;
        let len = 17u8;

        buf.push(len);
        buf.push(scale.to_le_bytes()[0]);
        buf.extend(&bytes);
        buf.extend(&[0u8; 8]);

        IsNull::No
    }
}

impl Decode<'_, Mssql> for u64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        if bytes.len() < 17 {
            return Err("Invalid numeric value length".into());
        }

        let value_bytes = &bytes[2..10];
        Ok(u64::from_le_bytes(value_bytes.try_into()?))
    }
}
