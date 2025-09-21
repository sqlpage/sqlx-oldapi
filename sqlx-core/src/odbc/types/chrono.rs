use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::types::Type;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use odbc_api::DataType;

impl Type<Odbc> for NaiveDate {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::DATE
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Date)
            || ty.data_type().accepts_character_data()
            || ty.data_type().accepts_numeric_data()
            || matches!(ty.data_type(), DataType::Other { .. } | DataType::Unknown)
    }
}

impl Type<Odbc> for NaiveTime {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIME
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Time { .. })
            || ty.data_type().accepts_character_data()
            || ty.data_type().accepts_numeric_data()
            || matches!(ty.data_type(), DataType::Other { .. } | DataType::Unknown)
    }
}

impl Type<Odbc> for NaiveDateTime {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIMESTAMP
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Timestamp { .. })
            || ty.data_type().accepts_character_data()
            || ty.data_type().accepts_numeric_data()
            || matches!(ty.data_type(), DataType::Other { .. } | DataType::Unknown)
    }
}

impl Type<Odbc> for DateTime<Utc> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIMESTAMP
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Timestamp { .. })
            || ty.data_type().accepts_character_data()
            || ty.data_type().accepts_numeric_data()
            || matches!(ty.data_type(), DataType::Other { .. } | DataType::Unknown)
    }
}

impl Type<Odbc> for DateTime<FixedOffset> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIMESTAMP
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Timestamp { .. })
            || ty.data_type().accepts_character_data()
            || ty.data_type().accepts_numeric_data()
            || matches!(ty.data_type(), DataType::Other { .. } | DataType::Unknown)
    }
}

impl Type<Odbc> for DateTime<Local> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIMESTAMP
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Timestamp { .. })
            || ty.data_type().accepts_character_data()
            || matches!(ty.data_type(), DataType::Other { .. } | DataType::Unknown)
    }
}

impl<'q> Encode<'q, Odbc> for NaiveDate {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.format("%Y-%m-%d").to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.format("%Y-%m-%d").to_string()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for NaiveTime {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.format("%H:%M:%S").to_string()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.format("%H:%M:%S").to_string()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for NaiveDateTime {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(
            self.format("%Y-%m-%d %H:%M:%S").to_string(),
        ));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(
            self.format("%Y-%m-%d %H:%M:%S").to_string(),
        ));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for DateTime<Utc> {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(
            self.format("%Y-%m-%d %H:%M:%S").to_string(),
        ));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(
            self.format("%Y-%m-%d %H:%M:%S").to_string(),
        ));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for DateTime<FixedOffset> {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(
            self.format("%Y-%m-%d %H:%M:%S").to_string(),
        ));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(
            self.format("%Y-%m-%d %H:%M:%S").to_string(),
        ));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for DateTime<Local> {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(
            self.format("%Y-%m-%d %H:%M:%S").to_string(),
        ));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(
            self.format("%Y-%m-%d %H:%M:%S").to_string(),
        ));
        crate::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Odbc> for NaiveDate {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        // Accept YYYYMMDD (some SQLite ODBC configs) as a date as well
        if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
            if let (Ok(y), Ok(m), Ok(d)) = (
                s[0..4].parse::<i32>(),
                s[4..6].parse::<u32>(),
                s[6..8].parse::<u32>(),
            ) {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Ok(date);
                }
            }
        }
        Ok(s.parse()?)
    }
}

impl<'r> Decode<'r, Odbc> for NaiveTime {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        Ok(s.parse()?)
    }
}

impl<'r> Decode<'r, Odbc> for NaiveDateTime {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut s = <String as Decode<'r, Odbc>>::decode(value)?;
        // Some ODBC drivers (e.g. PostgreSQL) may include trailing spaces or NULs
        // in textual representations of timestamps. Trim them before parsing.
        if s.ends_with('\u{0}') {
            s = s.trim_end_matches('\u{0}').to_string();
        }
        let s_trimmed = s.trim();
        // Try strict format first, then fall back to Chrono's FromStr
        if let Ok(dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S") {
            return Ok(dt);
        }
        Ok(s_trimmed.parse()?)
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<Utc> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        let s_trimmed = s.trim();

        // First try to parse as a UTC timestamp with timezone
        if let Ok(dt) = s_trimmed.parse::<DateTime<Utc>>() {
            return Ok(dt);
        }

        // If that fails, try to parse as a naive datetime and convert to UTC
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S") {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
        }

        // Finally, try chrono's default naive datetime parser
        if let Ok(naive_dt) = s_trimmed.parse::<NaiveDateTime>() {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
        }

        Err(format!("Cannot parse '{}' as DateTime<Utc>", s_trimmed).into())
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<FixedOffset> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        let s_trimmed = s.trim();

        // First try to parse as a timestamp with timezone/offset
        if let Ok(dt) = s_trimmed.parse::<DateTime<FixedOffset>>() {
            return Ok(dt);
        }

        // If that fails, try to parse as a naive datetime and assume UTC (zero offset)
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S") {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).fixed_offset());
        }

        // Finally, try chrono's default naive datetime parser
        if let Ok(naive_dt) = s_trimmed.parse::<NaiveDateTime>() {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).fixed_offset());
        }

        Err(format!("Cannot parse '{}' as DateTime<FixedOffset>", s_trimmed).into())
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<Local> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        Ok(s.parse::<DateTime<Utc>>()?.with_timezone(&Local))
    }
}
