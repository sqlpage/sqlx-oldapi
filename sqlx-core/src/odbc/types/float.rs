use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use odbc_api::DataType;

impl Type<Odbc> for f64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::DOUBLE
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Double
                | DataType::Float { .. }
                | DataType::Real
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
                | DataType::Integer
                | DataType::BigInt
                | DataType::SmallInt
                | DataType::TinyInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl Type<Odbc> for f32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::float(24) // Standard float precision
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(
            ty.data_type(),
            DataType::Float { .. }
                | DataType::Real
                | DataType::Double
                | DataType::Numeric { .. }
                | DataType::Decimal { .. }
                | DataType::Integer
                | DataType::BigInt
                | DataType::SmallInt
                | DataType::TinyInt
        ) || ty.data_type().accepts_character_data() // Allow parsing from strings
    }
}

impl<'q> Encode<'q, Odbc> for f32 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Float(self as f64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Float(*self as f64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for f64 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Float(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Float(*self));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for f64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(f) = value.float {
            return Ok(f);
        }
        if let Some(bytes) = value.blob {
            let s = std::str::from_utf8(bytes)?;
            return Ok(s.trim().parse()?);
        }
        Err("ODBC: cannot decode f64".into())
    }
}

impl<'r> Decode<'r, Odbc> for f32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(<f64 as Decode<'r, Odbc>>::decode(value)? as f32)
    }
}
