use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use odbc_api::DataType;

impl Type<Odbc> for i32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::INTEGER
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Integer | DataType::SmallInt | DataType::TinyInt | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for i64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIGINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::BigInt
                | DataType::Integer
                | DataType::SmallInt
                | DataType::TinyInt
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for i16 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::SMALLINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::SmallInt | DataType::TinyInt | DataType::Integer | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for i8 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TINYINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::TinyInt | DataType::SmallInt | DataType::Integer | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u8 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TINYINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::TinyInt | DataType::SmallInt | DataType::Integer | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u16 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::SMALLINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::SmallInt | DataType::Integer | DataType::BigInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::INTEGER
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Integer | DataType::BigInt)
            || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for u64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::BIGINT
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::BigInt
                | DataType::Integer
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl<'q> Encode<'q, Odbc> for i32 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i64 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i16 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i8 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u8 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u16 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u32 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u64 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        match i64::try_from(self) {
            Ok(value) => {
                buf.push(OdbcArgumentValue::Int(value));
                crate::encode::IsNull::No
            }
            Err(_) => {
                log::warn!("u64 value {} too large for ODBC, encoding as NULL", self);
                buf.push(OdbcArgumentValue::Null);
                crate::encode::IsNull::Yes
            }
        }
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        match i64::try_from(*self) {
            Ok(value) => {
                buf.push(OdbcArgumentValue::Int(value));
                crate::encode::IsNull::No
            }
            Err(_) => {
                log::warn!("u64 value {} too large for ODBC, encoding as NULL", self);
                buf.push(OdbcArgumentValue::Null);
                crate::encode::IsNull::Yes
            }
        }
    }
}

impl<'r> Decode<'r, Odbc> for i64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(i) = value.int {
            return Ok(i);
        }
        if let Some(bytes) = value.blob {
            let s = std::str::from_utf8(bytes)?;
            return Ok(s.trim().parse()?);
        }
        Err("ODBC: cannot decode i64".into())
    }
}

impl<'r> Decode<'r, Odbc> for i32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(i64::decode(value)? as i32)
    }
}

impl<'r> Decode<'r, Odbc> for i16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(i64::decode(value)? as i16)
    }
}

impl<'r> Decode<'r, Odbc> for i8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(i64::decode(value)? as i8)
    }
}

impl<'r> Decode<'r, Odbc> for u8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = i64::decode(value)?;
        Ok(u8::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = i64::decode(value)?;
        Ok(u16::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = i64::decode(value)?;
        Ok(u32::try_from(i)?)
    }
}

impl<'r> Decode<'r, Odbc> for u64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let i = i64::decode(value)?;
        Ok(u64::try_from(i)?)
    }
}
