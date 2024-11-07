use byteorder::{BigEndian, ByteOrder};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;

impl Type<Postgres> for i8 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::CHAR
    }
}

impl PgHasArrayType for i8 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::CHAR_ARRAY
    }
}

impl Encode<'_, Postgres> for i8 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i8 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        // note: in the TEXT encoding, a value of "0" here is encoded as an empty string
        let bytes = value.as_bytes()?;
        let bytes: [u8; 1] = bytes
            .get(..1)
            .unwrap_or(&[0])
            .try_into()
            .unwrap_or_default();
        Ok(i8::from_be_bytes(bytes))
    }
}

impl Type<Postgres> for i16 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT2
    }
}

impl PgHasArrayType for i16 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INT2_ARRAY
    }
}

impl Encode<'_, Postgres> for i16 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i16 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i16(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for i32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT4
    }
}

impl PgHasArrayType for i32 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INT4_ARRAY
    }
}

impl Encode<'_, Postgres> for i32 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i32 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for i64 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT8
    }
}

impl PgHasArrayType for i64 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INT8_ARRAY
    }
}

impl Encode<'_, Postgres> for i64 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i64 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i64(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for u16 {
    fn type_info() -> PgTypeInfo {
        <i32 as Type<Postgres>>::type_info()
    }
}

impl PgHasArrayType for u16 {
    fn array_type_info() -> PgTypeInfo {
        <i32 as PgHasArrayType>::array_type_info()
    }
}

impl Encode<'_, Postgres> for u16 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let v = i32::from(*self);
        <i32 as Encode<'_, Postgres>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Postgres> for u16 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = <i32 as Decode<'_, Postgres>>::decode(value)?;
        Ok(u16::try_from(v)?)
    }
}

impl Type<Postgres> for u32 {
    fn type_info() -> PgTypeInfo {
        <i64 as Type<Postgres>>::type_info()
    }
}

impl PgHasArrayType for u32 {
    fn array_type_info() -> PgTypeInfo {
        <i64 as PgHasArrayType>::array_type_info()
    }
}

impl Encode<'_, Postgres> for u32 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let v = i64::from(*self);
        <i64 as Encode<'_, Postgres>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Postgres> for u32 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = <i64 as Decode<'_, Postgres>>::decode(value)?;
        Ok(u32::try_from(v)?)
    }
}

impl Type<Postgres> for u64 {
    fn type_info() -> PgTypeInfo {
        <i64 as Type<Postgres>>::type_info()
    }
}

impl PgHasArrayType for u64 {
    fn array_type_info() -> PgTypeInfo {
        <i64 as PgHasArrayType>::array_type_info()
    }
}

impl Encode<'_, Postgres> for u64 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let v = i64::try_from(*self).unwrap_or_else(|_| {
            let v = i64::MAX;
            log::warn!("cannot encode {self} as a signed postgres bigint, encoding as {v} instead");
            v
        });
        <i64 as Encode<'_, Postgres>>::encode_by_ref(&v, buf)
    }
}

impl Decode<'_, Postgres> for u64 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = <i64 as Decode<'_, Postgres>>::decode(value)?;
        Ok(u64::try_from(v)?)
    }
}
