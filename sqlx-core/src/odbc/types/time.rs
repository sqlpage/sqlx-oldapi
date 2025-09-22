use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

impl Type<Odbc> for OffsetDateTime {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::timestamp(6)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_datetime_data() || ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for PrimitiveDateTime {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::timestamp(6)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_datetime_data() || ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for Date {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::new(odbc_api::DataType::Date)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_datetime_data() || ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for Time {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::time(6)
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        ty.data_type().accepts_datetime_data() || ty.data_type().accepts_character_data()
    }
}

impl<'q> Encode<'q, Odbc> for OffsetDateTime {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        let utc_dt = self.to_offset(time::UtcOffset::UTC);
        let primitive_dt = PrimitiveDateTime::new(utc_dt.date(), utc_dt.time());
        buf.push(OdbcArgumentValue::Text(primitive_dt.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        let utc_dt = self.to_offset(time::UtcOffset::UTC);
        let primitive_dt = PrimitiveDateTime::new(utc_dt.date(), utc_dt.time());
        buf.push(OdbcArgumentValue::Text(primitive_dt.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for PrimitiveDateTime {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for Date {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for Time {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_string()));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for OffsetDateTime {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.text {
            // Try parsing as ISO-8601 timestamp with timezone
            if let Ok(dt) = OffsetDateTime::parse(
                text,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return Ok(dt);
            }
            // Try parsing as primitive datetime and assume UTC
            if let Ok(dt) = PrimitiveDateTime::parse(
                text,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return Ok(dt.assume_utc());
            }
            // Try custom formats that ODBC might return
            if let Ok(dt) = time::PrimitiveDateTime::parse(
                text,
                &time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
            ) {
                return Ok(dt.assume_utc());
            }
        }
        Err("ODBC: cannot decode OffsetDateTime".into())
    }
}

impl<'r> Decode<'r, Odbc> for PrimitiveDateTime {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.text {
            // Try parsing as ISO-8601
            if let Ok(dt) = PrimitiveDateTime::parse(
                text,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return Ok(dt);
            }
            // Try custom formats that ODBC might return
            if let Ok(dt) = PrimitiveDateTime::parse(
                text,
                &time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
            ) {
                return Ok(dt);
            }
            if let Ok(dt) = PrimitiveDateTime::parse(
                text,
                &time::macros::format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"
                ),
            ) {
                return Ok(dt);
            }
        }
        Err("ODBC: cannot decode PrimitiveDateTime".into())
    }
}

impl<'r> Decode<'r, Odbc> for Date {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.text {
            if let Ok(date) = Date::parse(
                text,
                &time::macros::format_description!("[year]-[month]-[day]"),
            ) {
                return Ok(date);
            }
            if let Ok(date) = Date::parse(
                text,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return Ok(date);
            }
        }
        Err("ODBC: cannot decode Date".into())
    }
}

impl<'r> Decode<'r, Odbc> for Time {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.text {
            if let Ok(time) = Time::parse(
                text,
                &time::macros::format_description!("[hour]:[minute]:[second]"),
            ) {
                return Ok(time);
            }
            if let Ok(time) = Time::parse(
                text,
                &time::macros::format_description!("[hour]:[minute]:[second].[subsecond]"),
            ) {
                return Ok(time);
            }
        }
        Err("ODBC: cannot decode Time".into())
    }
}
