use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;

impl Type<Odbc> for Vec<u8> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varbinary(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_binary_data() || ty.data_type().accepts_character_data()
        // Allow decoding from character types too
    }
}

impl<'q> Encode<'q, Odbc> for Vec<u8> {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.clone()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for &'q [u8] {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.to_vec()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.to_vec()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for Vec<u8> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(bytes) = value.blob {
            return Ok(bytes.to_vec());
        }
        if let Some(text) = value.text {
            return Ok(text.as_bytes().to_vec());
        }
        Err("ODBC: cannot decode Vec<u8>".into())
    }
}

impl<'r> Decode<'r, Odbc> for &'r [u8] {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(bytes) = value.blob {
            return Ok(bytes);
        }
        if let Some(text) = value.text {
            return Ok(text.as_bytes());
        }
        Err("ODBC: cannot decode &[u8]".into())
    }
}

impl Type<Odbc> for [u8] {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varbinary(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_binary_data() || ty.data_type().accepts_character_data()
    }
}
