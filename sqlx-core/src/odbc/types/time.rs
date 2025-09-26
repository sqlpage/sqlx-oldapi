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
        ty.data_type().accepts_datetime_data()
            || ty.data_type().accepts_character_data()
            || ty.data_type().accepts_numeric_data() // For Unix timestamps
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

// Helper function for parsing datetime from Unix timestamp
fn parse_unix_timestamp_as_offset_datetime(timestamp: i64) -> Option<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp(timestamp).ok()
}

impl<'r> Decode<'r, Odbc> for OffsetDateTime {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle numeric timestamps (Unix epoch seconds) first
        if let Some(int_val) = value.int() {
            if let Some(dt) = parse_unix_timestamp_as_offset_datetime(int_val) {
                return Ok(dt);
            }
        }

        if let Some(float_val) = value.float::<f64>() {
            if let Some(dt) = parse_unix_timestamp_as_offset_datetime(float_val as i64) {
                return Ok(dt);
            }
        }

        // Handle text values
        if let Some(text) = value.text() {
            let trimmed = text.trim();
            // Try parsing as ISO-8601 timestamp with timezone
            if let Ok(dt) = OffsetDateTime::parse(
                trimmed,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return Ok(dt);
            }
            // Try parsing as primitive datetime and assume UTC
            if let Ok(dt) = PrimitiveDateTime::parse(
                trimmed,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return Ok(dt.assume_utc());
            }
            // Try custom formats that ODBC might return
            if let Ok(dt) = time::PrimitiveDateTime::parse(
                trimmed,
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
        // Handle numeric timestamps (Unix epoch seconds) first
        if let Some(int_val) = value.int() {
            if let Some(offset_dt) = parse_unix_timestamp_as_offset_datetime(int_val) {
                let utc_dt = offset_dt.to_offset(time::UtcOffset::UTC);
                return Ok(PrimitiveDateTime::new(utc_dt.date(), utc_dt.time()));
            }
        }

        if let Some(float_val) = value.float::<f64>() {
            if let Some(offset_dt) = parse_unix_timestamp_as_offset_datetime(float_val as i64) {
                let utc_dt = offset_dt.to_offset(time::UtcOffset::UTC);
                return Ok(PrimitiveDateTime::new(utc_dt.date(), utc_dt.time()));
            }
        }

        // Handle text values
        if let Some(text) = value.text() {
            let trimmed = text.trim();
            // Try parsing as ISO-8601
            if let Ok(dt) = PrimitiveDateTime::parse(
                trimmed,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return Ok(dt);
            }
            // Try custom formats that ODBC might return
            if let Ok(dt) = PrimitiveDateTime::parse(
                trimmed,
                &time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
            ) {
                return Ok(dt);
            }
            if let Ok(dt) = PrimitiveDateTime::parse(
                trimmed,
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

// Helper functions for time crate date parsing
fn parse_yyyymmdd_as_time_date(val: i64) -> Option<Date> {
    if (19000101..=30001231).contains(&val) {
        let year = (val / 10000) as i32;
        let month = ((val % 10000) / 100) as u8;
        let day = (val % 100) as u8;

        if let Ok(month_enum) = time::Month::try_from(month) {
            Date::from_calendar_date(year, month_enum, day).ok()
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_yyyymmdd_text_as_time_date(s: &str) -> Option<Date> {
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        if let (Ok(y), Ok(m), Ok(d)) = (
            s[0..4].parse::<i32>(),
            s[4..6].parse::<u8>(),
            s[6..8].parse::<u8>(),
        ) {
            if let Ok(month_enum) = time::Month::try_from(m) {
                return Date::from_calendar_date(y, month_enum, d).ok();
            }
        }
    }
    None
}

impl<'r> Decode<'r, Odbc> for Date {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle raw ODBC Date values first
        if let Some(date_val) = value.date() {
            // Convert odbc_api::sys::Date to time::Date
            // The ODBC Date structure typically has year, month, day fields
            let month = time::Month::try_from(date_val.month as u8)
                .map_err(|_| "ODBC: invalid month value")?;
            return Ok(Date::from_calendar_date(
                date_val.year as i32,
                month,
                date_val.day as u8,
            )?);
        }

        // Handle numeric YYYYMMDD format first
        if let Some(int_val) = value.int() {
            if let Some(date) = parse_yyyymmdd_as_time_date(int_val) {
                return Ok(date);
            }

            // Fallback: try as days since Unix epoch
            if let Ok(days) = i32::try_from(int_val) {
                let epoch = Date::from_calendar_date(1970, time::Month::January, 1)?;
                if let Some(date) = epoch.checked_add(time::Duration::days(days as i64)) {
                    return Ok(date);
                }
            }
        }

        // Handle float values
        if let Some(float_val) = value.float::<f64>() {
            if let Some(date) = parse_yyyymmdd_as_time_date(float_val as i64) {
                return Ok(date);
            }
        }

        // Handle text values
        if let Some(text) = value.text() {
            let trimmed = text.trim();
            if let Some(date) = parse_yyyymmdd_text_as_time_date(trimmed) {
                return Ok(date);
            }

            if let Ok(date) = Date::parse(
                trimmed,
                &time::macros::format_description!("[year]-[month]-[day]"),
            ) {
                return Ok(date);
            }
            if let Ok(date) = Date::parse(
                trimmed,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return Ok(date);
            }
        }

        Err("ODBC: cannot decode Date".into())
    }
}

// Helper function for time parsing from seconds since midnight
fn parse_seconds_as_time(seconds: i64) -> Option<Time> {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if (0..24).contains(&hours) && (0..60).contains(&minutes) && (0..60).contains(&secs) {
        Time::from_hms(hours as u8, minutes as u8, secs as u8).ok()
    } else {
        None
    }
}

impl<'r> Decode<'r, Odbc> for Time {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle numeric time (seconds since midnight)
        if let Some(int_val) = value.int::<i64>() {
            if let Some(time) = parse_seconds_as_time(int_val) {
                return Ok(time);
            }
        }

        if let Some(float_val) = value.float::<f64>() {
            if let Some(time) = parse_seconds_as_time(float_val as i64) {
                return Ok(time);
            }
        }

        // Handle text values
        if let Some(text) = value.text() {
            let trimmed = text.trim();
            if let Ok(time) = Time::parse(
                trimmed,
                &time::macros::format_description!("[hour]:[minute]:[second]"),
            ) {
                return Ok(time);
            }
            if let Ok(time) = Time::parse(
                trimmed,
                &time::macros::format_description!("[hour]:[minute]:[second].[subsecond]"),
            ) {
                return Ok(time);
            }
        }

        Err("ODBC: cannot decode Time".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odbc::{ColumnData, OdbcTypeInfo, OdbcValueRef, OdbcValueVec};
    use odbc_api::DataType;
    use time::{macros::date, macros::time as time_macro};

    fn create_test_value_text(text: &str, data_type: DataType) -> OdbcValueRef<'static> {
        let column = ColumnData {
            values: OdbcValueVec::Text(vec![Some(text.to_string())]),
            type_info: OdbcTypeInfo::new(data_type),
        };
        let ptr = Box::leak(Box::new(column));
        OdbcValueRef::new(ptr, 0)
    }

    fn create_test_value_int(value: i64, data_type: DataType) -> OdbcValueRef<'static> {
        let column = ColumnData {
            values: OdbcValueVec::NullableBigInt(vec![Some(value)]),
            type_info: OdbcTypeInfo::new(data_type),
        };
        let ptr = Box::leak(Box::new(column));
        OdbcValueRef::new(ptr, 0)
    }

    #[test]
    fn test_offset_datetime_type_compatibility() {
        assert!(<OffsetDateTime as Type<Odbc>>::compatible(
            &OdbcTypeInfo::TIMESTAMP
        ));
        assert!(<OffsetDateTime as Type<Odbc>>::compatible(
            &OdbcTypeInfo::varchar(None)
        ));
        assert!(<OffsetDateTime as Type<Odbc>>::compatible(
            &OdbcTypeInfo::DOUBLE
        ));
    }

    #[test]
    fn test_primitive_datetime_decode_from_text() -> Result<(), BoxDynError> {
        let value =
            create_test_value_text("2023-12-25 14:30:00", DataType::Timestamp { precision: 0 });
        let decoded = <PrimitiveDateTime as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded.year(), 2023);
        assert_eq!(decoded.month() as u8, 12);
        assert_eq!(decoded.day(), 25);
        assert_eq!(decoded.hour(), 14);
        assert_eq!(decoded.minute(), 30);
        assert_eq!(decoded.second(), 0);

        Ok(())
    }

    #[test]
    fn test_date_decode_from_text() -> Result<(), BoxDynError> {
        let value = create_test_value_text("2023-12-25", DataType::Date);
        let decoded = <Date as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded.year(), 2023);
        assert_eq!(decoded.month() as u8, 12);
        assert_eq!(decoded.day(), 25);

        Ok(())
    }

    #[test]
    fn test_time_decode_from_text() -> Result<(), BoxDynError> {
        let value = create_test_value_text("14:30:00", DataType::Time { precision: 0 });
        let decoded = <Time as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded.hour(), 14);
        assert_eq!(decoded.minute(), 30);
        assert_eq!(decoded.second(), 0);

        Ok(())
    }

    #[test]
    fn test_encode_date() {
        let mut buf = Vec::new();
        let date = date!(2023 - 12 - 25);
        let result = <Date as Encode<Odbc>>::encode(date, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Text(text) = &buf[0] {
            assert_eq!(text, "2023-12-25");
        } else {
            panic!("Expected Text argument");
        }
    }

    #[test]
    fn test_encode_time() {
        let mut buf = Vec::new();
        let time = time_macro!(14:30:00);
        let result = <Time as Encode<Odbc>>::encode(time, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(buf.len(), 1);
        if let OdbcArgumentValue::Text(text) = &buf[0] {
            // Accept both "14:30:00" and "14:30:00.0" since Time.to_string() may include microseconds
            assert!(
                text == "14:30:00" || text == "14:30:00.0",
                "Expected '14:30:00' or '14:30:00.0', got '{}'",
                text
            );
        } else {
            panic!("Expected Text argument");
        }
    }

    #[test]
    fn test_decode_error_handling() {
        let column = ColumnData {
            values: OdbcValueVec::Text(vec![Some("not_a_datetime".to_string())]),
            type_info: OdbcTypeInfo::TIMESTAMP,
        };
        let ptr = Box::leak(Box::new(column));
        let value = OdbcValueRef::new(ptr, 0);

        let result = <PrimitiveDateTime as Decode<Odbc>>::decode(value);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "ODBC: cannot decode PrimitiveDateTime"
        );
    }
}
