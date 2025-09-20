use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use std::str::FromStr;
use uuid::Uuid;

impl Type<Odbc> for Uuid {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varchar(Some(std::num::NonZeroUsize::new(36).unwrap()))
        // UUID string length
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_character_data() || ty.data_type().accepts_binary_data()
    }
}

impl<'q> Encode<'q, Odbc> for Uuid {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for Uuid {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(bytes) = value.blob {
            if bytes.len() == 16 {
                // Binary UUID format
                return Ok(Uuid::from_bytes(bytes.try_into()?));
            }
            // Try as string
            let s = std::str::from_utf8(bytes)?;
            return Ok(Uuid::from_str(s)?);
        }
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        Ok(Uuid::from_str(&s)?)
    }
}
