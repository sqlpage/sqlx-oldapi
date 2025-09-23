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
                let mut uuid_bytes = [0u8; 16];
                for (i, chunk) in bytes.chunks(8).enumerate() {
                    if i >= 16 {
                        break;
                    }
                    let mut byte_val = 0u8;
                    for (j, &bit_byte) in chunk.iter().enumerate() {
                        if bit_byte == 49 {
                            byte_val |= 1 << (7 - j);
                        }
                    }
                    uuid_bytes[i] = byte_val;
                }
                return Ok(Uuid::from_bytes(uuid_bytes));
            }
            // Some drivers may return UUIDs as ASCII/UTF-8 bytes
            let s = std::str::from_utf8(bytes)?;
            let s = s.trim_matches('\u{0}').trim();
            let s = if s.len() > 3 && (s.starts_with("X'") || s.starts_with("x'")) && s.ends_with("'") {
                &s[2..s.len() - 1]
            } else {
                s
            };
            // If it's 32 hex digits without dashes, accept it
            if s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit()) {
                let mut buf = [0u8; 16];
                for i in 0..16 {
                    let byte_str = &s[i * 2..i * 2 + 2];
                    buf[i] = u8::from_str_radix(byte_str, 16)?;
                }
                return Ok(Uuid::from_bytes(buf));
            }
            return Ok(Uuid::from_str(s).map_err(|e| format!("Invalid UUID: {}, error: {}", s, e))?);
        }
        let mut s = <String as Decode<'r, Odbc>>::decode(value)?;
        if s.ends_with('\u{0}') {
            s = s.trim_end_matches('\u{0}').to_string();
        }
        let s = s.trim();
        let s = if s.len() > 3 && (s.starts_with("X'") || s.starts_with("x'")) && s.ends_with("'") {
            &s[2..s.len() - 1]
        } else {
            s
        };
        if s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            let mut buf = [0u8; 16];
            for i in 0..16 {
                let byte_str = &s[i * 2..i * 2 + 2];
                buf[i] = u8::from_str_radix(byte_str, 16)?;
            }
            return Ok(Uuid::from_bytes(buf));
        }
        Ok(Uuid::from_str(s).map_err(|e| format!("Invalid UUID: {}, error: {}", s, e))?)
    }
}
