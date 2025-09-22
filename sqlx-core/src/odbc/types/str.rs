use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;

impl Type<Odbc> for str {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varchar(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for String {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varchar(None)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_character_data()
    }
}

impl<'q> Encode<'q, Odbc> for String {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.clone()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for &'q str {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_owned()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text((*self).to_owned()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for String {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.text {
            return Ok(text.to_owned());
        }
        if let Some(bytes) = value.blob {
            return Ok(std::str::from_utf8(bytes)?.to_owned());
        }
        Err("ODBC: cannot decode String".into())
    }
}

impl<'r> Decode<'r, Odbc> for &'r str {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.text {
            return Ok(text);
        }
        if let Some(bytes) = value.blob {
            return Ok(std::str::from_utf8(bytes)?);
        }
        Err("ODBC: cannot decode &str".into())
    }
}
