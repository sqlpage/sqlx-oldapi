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
        ty.data_type().accepts_character_data()
            || ty.data_type().accepts_binary_data()
            || matches!(
                ty.data_type(),
                odbc_api::DataType::Other { .. } | odbc_api::DataType::Unknown
            )
    }
}

impl<'q> Encode<'q, Odbc> for Uuid {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for Uuid {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(bytes) = value.blob {
            if bytes.len() == 16 {
                return Ok(Uuid::from_bytes(bytes.try_into()?));
            } else if bytes.len() == 128 {
                // Each byte is ASCII '0' or '1' representing a bit
                let mut uuid_bytes = [0u8; 16];
                for (i, chunk) in bytes.chunks(8).enumerate() {
                    if i >= 16 {
                        break;
                    }
                    let mut byte_val = 0u8;
                    for (j, &bit_byte) in chunk.iter().enumerate() {
                        if bit_byte == 49 {
                            // ASCII '1'
                            byte_val |= 1 << (7 - j);
                        }
                    }
                    uuid_bytes[i] = byte_val;
                }
                return Ok(Uuid::from_bytes(uuid_bytes));
            }
            // Some drivers may return UUIDs as ASCII/UTF-8 bytes
            let s = std::str::from_utf8(bytes)?.trim();
            return Ok(Uuid::from_str(s).map_err(|e| format!("Invalid UUID: {}, error: {}", s, e))?);
        }
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        Ok(Uuid::from_str(s.trim()).map_err(|e| format!("Invalid UUID: {}, error: {}", s, e))?)
    }
}
