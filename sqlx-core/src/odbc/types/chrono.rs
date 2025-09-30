use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::BoxDynError;
use crate::odbc::{DataTypeExt, Odbc, OdbcArgumentValue, OdbcTypeInfo, OdbcValueRef};
use crate::type_info::TypeInfo;
use crate::types::Type;
use chrono::{
    DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc,
};
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
        buf.push(OdbcArgumentValue::Date(odbc_api::sys::Date {
            year: self.year() as i16,
            month: self.month() as u16,
            day: self.day() as u16,
        }));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Date(odbc_api::sys::Date {
            year: self.year() as i16,
            month: self.month() as u16,
            day: self.day() as u16,
        }));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for NaiveTime {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Time(odbc_api::sys::Time {
            hour: self.hour() as u16,
            minute: self.minute() as u16,
            second: self.second() as u16,
        }));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Time(odbc_api::sys::Time {
            hour: self.hour() as u16,
            minute: self.minute() as u16,
            second: self.second() as u16,
        }));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for NaiveDateTime {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Timestamp(odbc_api::sys::Timestamp {
            year: self.year() as i16,
            month: self.month() as u16,
            day: self.day() as u16,
            hour: self.hour() as u16,
            minute: self.minute() as u16,
            second: self.second() as u16,
            fraction: self.nanosecond(),
        }));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Timestamp(odbc_api::sys::Timestamp {
            year: self.year() as i16,
            month: self.month() as u16,
            day: self.day() as u16,
            hour: self.hour() as u16,
            minute: self.minute() as u16,
            second: self.second() as u16,
            fraction: self.nanosecond(),
        }));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for DateTime<Utc> {
    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_rfc3339()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for DateTime<FixedOffset> {
    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_rfc3339()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for DateTime<Local> {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        let naive = self.naive_local();
        buf.push(OdbcArgumentValue::Timestamp(odbc_api::sys::Timestamp {
            year: naive.year() as i16,
            month: naive.month() as u16,
            day: naive.day() as u16,
            hour: naive.hour() as u16,
            minute: naive.minute() as u16,
            second: naive.second() as u16,
            fraction: naive.nanosecond(),
        }));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue>) -> crate::encode::IsNull {
        let naive = self.naive_local();
        buf.push(OdbcArgumentValue::Timestamp(odbc_api::sys::Timestamp {
            year: naive.year() as i16,
            month: naive.month() as u16,
            day: naive.day() as u16,
            hour: naive.hour() as u16,
            minute: naive.minute() as u16,
            second: naive.second() as u16,
            fraction: naive.nanosecond(),
        }));
        crate::encode::IsNull::No
    }
}

// Helper functions for date parsing
fn parse_yyyymmdd_as_naive_date(val: i64) -> Option<NaiveDate> {
    if (19000101..=30001231).contains(&val) {
        let year = (val / 10000) as i32;
        let month = ((val % 10000) / 100) as u32;
        let day = (val % 100) as u32;
        NaiveDate::from_ymd_opt(year, month, day)
    } else {
        None
    }
}

fn parse_yyyymmdd_text_as_naive_date(s: &str) -> Option<NaiveDate> {
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        if let (Ok(y), Ok(m), Ok(d)) = (
            s[0..4].parse::<i32>(),
            s[4..6].parse::<u32>(),
            s[6..8].parse::<u32>(),
        ) {
            return NaiveDate::from_ymd_opt(y, m, d);
        }
    }
    None
}

fn get_text_from_value(value: &OdbcValueRef<'_>) -> Result<Option<String>, BoxDynError> {
    if let Some(text) = value.text() {
        let trimmed = text.trim_matches('\u{0}').trim();
        return Ok(Some(trimmed.to_string()));
    }
    if let Some(bytes) = value.blob() {
        let s = std::str::from_utf8(bytes)?;
        let trimmed = s.trim_matches('\u{0}').trim();
        return Ok(Some(trimmed.to_string()));
    }
    Ok(None)
}

impl<'r> Decode<'r, Odbc> for NaiveDate {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle raw ODBC Date values first
        if let Some(date_val) = value.date() {
            // Convert odbc_api::sys::Date to NaiveDate
            // The ODBC Date structure typically has year, month, day fields
            return Ok(NaiveDate::from_ymd_opt(
                date_val.year as i32,
                date_val.month as u32,
                date_val.day as u32,
            )
            .ok_or_else(|| "ODBC: invalid date values".to_string())?);
        }

        // Handle text values first (most common for dates)
        if let Some(text) = value.text() {
            if let Some(date) = parse_yyyymmdd_text_as_naive_date(text) {
                return Ok(date);
            }
            if let Ok(date) = text.parse() {
                return Ok(date);
            }
        }

        // Handle numeric YYYYMMDD format (for databases that return as numbers)
        if let Some(int_val) = value.int() {
            if let Some(date) = parse_yyyymmdd_as_naive_date(int_val) {
                return Ok(date);
            }
            return Err(format!(
                "ODBC: cannot decode NaiveDate from integer '{}': not in YYYYMMDD range",
                int_val
            )
            .into());
        }

        // Handle float values similarly
        if let Some(float_val) = value.float::<f64>() {
            if let Some(date) = parse_yyyymmdd_as_naive_date(float_val as i64) {
                return Ok(date);
            }
            return Err(format!(
                "ODBC: cannot decode NaiveDate from float '{}': not in YYYYMMDD range",
                float_val
            )
            .into());
        }

        Err(format!(
            "ODBC: cannot decode NaiveDate from value with type '{}'",
            value.batch.columns[value.column_index].type_info.name()
        )
        .into())
    }
}

impl<'r> Decode<'r, Odbc> for NaiveTime {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle raw ODBC Time values first
        if let Some(time_val) = value.time() {
            // Convert odbc_api::sys::Time to NaiveTime
            // The ODBC Time structure typically has hour, minute, second fields
            return Ok(NaiveTime::from_hms_opt(
                time_val.hour as u32,
                time_val.minute as u32,
                time_val.second as u32,
            )
            .ok_or_else(|| "ODBC: invalid time values".to_string())?);
        }

        let mut s = <String as Decode<'r, Odbc>>::decode(value)?;
        if s.ends_with('\u{0}') {
            s = s.trim_end_matches('\u{0}').to_string();
        }
        let s_trimmed = s.trim();
        Ok(s_trimmed
            .parse()
            .map_err(|e| format!("ODBC: cannot decode NaiveTime from '{}': {}", s_trimmed, e))?)
    }
}

impl<'r> Decode<'r, Odbc> for NaiveDateTime {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle raw ODBC Timestamp values first
        if let Some(ts_val) = value.timestamp() {
            // Convert odbc_api::sys::Timestamp to NaiveDateTime
            // The ODBC Timestamp structure typically has year, month, day, hour, minute, second fields
            let date =
                NaiveDate::from_ymd_opt(ts_val.year as i32, ts_val.month as u32, ts_val.day as u32)
                    .ok_or_else(|| "ODBC: invalid date values in timestamp".to_string())?;

            let time = NaiveTime::from_hms_opt(
                ts_val.hour as u32,
                ts_val.minute as u32,
                ts_val.second as u32,
            )
            .ok_or_else(|| "ODBC: invalid time values in timestamp".to_string())?;

            return Ok(NaiveDateTime::new(date, time));
        }

        let mut s = <String as Decode<'r, Odbc>>::decode(value)?;
        // Some ODBC drivers (e.g. PostgreSQL) may include trailing spaces or NULs
        // in textual representations of timestamps. Trim them before parsing.
        if s.ends_with('\u{0}') {
            s = s.trim_end_matches('\u{0}').to_string();
        }
        let s_trimmed = s.trim();

        if let Ok(dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S%.f") {
            return Ok(dt);
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S") {
            return Ok(dt);
        }
        Ok(s_trimmed.parse().map_err(|e| {
            format!(
                "ODBC: cannot decode NaiveDateTime from '{}': {}",
                s_trimmed, e
            )
        })?)
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<Utc> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle raw ODBC Timestamp values first
        if let Some(ts_val) = value.timestamp() {
            // Convert odbc_api::sys::Timestamp to DateTime<Utc>
            // The ODBC Timestamp structure typically has year, month, day, hour, minute, second fields
            let naive_dt = NaiveDateTime::new(
                NaiveDate::from_ymd_opt(ts_val.year as i32, ts_val.month as u32, ts_val.day as u32)
                    .ok_or_else(|| "ODBC: invalid date values in timestamp".to_string())?,
                NaiveTime::from_hms_opt(
                    ts_val.hour as u32,
                    ts_val.minute as u32,
                    ts_val.second as u32,
                )
                .ok_or_else(|| "ODBC: invalid time values in timestamp".to_string())?,
            );

            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
        }

        let mut s = <String as Decode<'r, Odbc>>::decode(value)?;
        if s.ends_with('\u{0}') {
            s = s.trim_end_matches('\u{0}').to_string();
        }
        let s_trimmed = s.trim();

        // First try to parse as a UTC timestamp with timezone
        if let Ok(dt) = s_trimmed.parse::<DateTime<Utc>>() {
            return Ok(dt);
        }

        // If that fails, try to parse as a naive datetime and convert to UTC
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S%.f") {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
        }
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S") {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
        }

        // Finally, try chrono's default naive datetime parser
        if let Ok(naive_dt) = s_trimmed.parse::<NaiveDateTime>() {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
        }

        Err(format!("ODBC: cannot decode DateTime<Utc> from '{}'", s_trimmed).into())
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<FixedOffset> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle raw ODBC Timestamp values first
        if let Some(ts_val) = value.timestamp() {
            // Convert odbc_api::sys::Timestamp to DateTime<FixedOffset>
            // The ODBC Timestamp structure typically has year, month, day, hour, minute, second fields
            let naive_dt = NaiveDateTime::new(
                NaiveDate::from_ymd_opt(ts_val.year as i32, ts_val.month as u32, ts_val.day as u32)
                    .ok_or_else(|| "ODBC: invalid date values in timestamp".to_string())?,
                NaiveTime::from_hms_opt(
                    ts_val.hour as u32,
                    ts_val.minute as u32,
                    ts_val.second as u32,
                )
                .ok_or_else(|| "ODBC: invalid time values in timestamp".to_string())?,
            );

            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).fixed_offset());
        }

        let mut s = <String as Decode<'r, Odbc>>::decode(value)?;
        if s.ends_with('\u{0}') {
            s = s.trim_end_matches('\u{0}').to_string();
        }
        let s_trimmed = s.trim();

        // First try to parse as a timestamp with timezone/offset
        if let Ok(dt) = s_trimmed.parse::<DateTime<FixedOffset>>() {
            return Ok(dt);
        }

        // If that fails, try to parse as a naive datetime and assume UTC (zero offset)
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S%.f") {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).fixed_offset());
        }
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s_trimmed, "%Y-%m-%d %H:%M:%S") {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).fixed_offset());
        }

        // Finally, try chrono's default naive datetime parser
        if let Ok(naive_dt) = s_trimmed.parse::<NaiveDateTime>() {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).fixed_offset());
        }

        Err(format!(
            "ODBC: cannot decode DateTime<FixedOffset> from '{}'",
            s_trimmed
        )
        .into())
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<Local> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        // Handle raw ODBC Timestamp values first
        if let Some(ts_val) = value.timestamp() {
            // Convert odbc_api::sys::Timestamp to DateTime<Local>
            // The ODBC Timestamp structure typically has year, month, day, hour, minute, second fields
            let naive_dt = NaiveDateTime::new(
                NaiveDate::from_ymd_opt(ts_val.year as i32, ts_val.month as u32, ts_val.day as u32)
                    .ok_or_else(|| "ODBC: invalid date values in timestamp".to_string())?,
                NaiveTime::from_hms_opt(
                    ts_val.hour as u32,
                    ts_val.minute as u32,
                    ts_val.second as u32,
                )
                .ok_or_else(|| "ODBC: invalid time values in timestamp".to_string())?,
            );

            return Ok(
                DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc).with_timezone(&Local)
            );
        }

        let mut s = <String as Decode<'r, Odbc>>::decode(value)?;
        if s.ends_with('\u{0}') {
            s = s.trim_end_matches('\u{0}').to_string();
        }
        let s_trimmed = s.trim();
        Ok(s_trimmed
            .parse::<DateTime<Utc>>()
            .map_err(|e| {
                format!(
                    "ODBC: cannot decode DateTime<Local> from '{}' as DateTime<Utc>: {}",
                    s_trimmed, e
                )
            })?
            .with_timezone(&Local))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odbc::{
        ColumnData, OdbcBatch, OdbcColumn, OdbcTypeInfo, OdbcValueRef, OdbcValueVec,
    };
    use crate::type_info::TypeInfo;
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use odbc_api::DataType;
    use std::sync::Arc;

    fn make_ref(value_vec: OdbcValueVec, data_type: DataType) -> OdbcValueRef<'static> {
        let column = ColumnData {
            values: value_vec,
            type_info: OdbcTypeInfo::new(data_type),
            nulls: vec![false],
        };
        let column_data = vec![Arc::new(column)];
        let batch = OdbcBatch {
            columns: Arc::new([OdbcColumn {
                name: "test".to_string(),
                type_info: OdbcTypeInfo::new(data_type),
                ordinal: 0,
            }]),
            column_data,
        };
        let batch_ptr = Box::leak(Box::new(batch));
        OdbcValueRef::new(batch_ptr, 0, 0)
    }

    fn create_test_value_text(text: &'static str, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::Text(vec![text.to_string()]), data_type)
    }

    fn create_test_value_int(value: i64, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::BigInt(vec![value]), data_type)
    }

    fn create_test_value_float(value: f64, data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::Double(vec![value]), data_type)
    }

    fn create_test_value_blob(data: &'static [u8], data_type: DataType) -> OdbcValueRef<'static> {
        make_ref(OdbcValueVec::Binary(vec![data.to_vec()]), data_type)
    }

    #[test]
    fn test_naive_date_type_compatibility() {
        assert!(<NaiveDate as Type<Odbc>>::compatible(&OdbcTypeInfo::DATE));
        assert!(<NaiveDate as Type<Odbc>>::compatible(
            &OdbcTypeInfo::varchar(None)
        ));
        assert!(<NaiveDate as Type<Odbc>>::compatible(
            &OdbcTypeInfo::INTEGER
        ));
    }

    #[test]
    fn test_parse_yyyymmdd_as_naive_date() {
        // Valid dates
        assert_eq!(
            parse_yyyymmdd_as_naive_date(20200102),
            Some(NaiveDate::from_ymd_opt(2020, 1, 2).unwrap())
        );
        assert_eq!(
            parse_yyyymmdd_as_naive_date(19991231),
            Some(NaiveDate::from_ymd_opt(1999, 12, 31).unwrap())
        );

        // Invalid dates
        assert_eq!(parse_yyyymmdd_as_naive_date(20201301), None); // Invalid month
        assert_eq!(parse_yyyymmdd_as_naive_date(20200230), None); // Invalid day
        assert_eq!(parse_yyyymmdd_as_naive_date(123456), None); // Too short
    }

    #[test]
    fn test_parse_yyyymmdd_text_as_naive_date() {
        // Valid dates
        assert_eq!(
            parse_yyyymmdd_text_as_naive_date("20200102"),
            Some(NaiveDate::from_ymd_opt(2020, 1, 2).unwrap())
        );
        assert_eq!(
            parse_yyyymmdd_text_as_naive_date("19991231"),
            Some(NaiveDate::from_ymd_opt(1999, 12, 31).unwrap())
        );

        // Invalid formats
        assert_eq!(parse_yyyymmdd_text_as_naive_date("2020-01-02"), None); // Dashes
        assert_eq!(parse_yyyymmdd_text_as_naive_date("20201301"), None); // Invalid month
        assert_eq!(parse_yyyymmdd_text_as_naive_date("abcd1234"), None); // Non-numeric
    }

    #[test]
    fn test_naive_date_decode_from_text() -> Result<(), BoxDynError> {
        // Standard ISO format
        let value = create_test_value_text("2020-01-02", DataType::Date);
        let decoded = <NaiveDate as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, NaiveDate::from_ymd_opt(2020, 1, 2).unwrap());

        // YYYYMMDD format
        let value = create_test_value_text("20200102", DataType::Date);
        let decoded = <NaiveDate as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, NaiveDate::from_ymd_opt(2020, 1, 2).unwrap());

        Ok(())
    }

    #[test]
    fn test_naive_date_decode_from_int() -> Result<(), BoxDynError> {
        let value = create_test_value_int(20200102, DataType::Date);
        let decoded = <NaiveDate as Decode<Odbc>>::decode(value)?;
        assert_eq!(decoded, NaiveDate::from_ymd_opt(2020, 1, 2).unwrap());

        Ok(())
    }

    #[test]
    fn test_naive_datetime_decode() -> Result<(), BoxDynError> {
        let value =
            create_test_value_text("2020-01-02 15:30:45", DataType::Timestamp { precision: 0 });
        let decoded = <NaiveDateTime as Decode<Odbc>>::decode(value)?;
        let expected = NaiveDate::from_ymd_opt(2020, 1, 2)
            .unwrap()
            .and_hms_opt(15, 30, 45)
            .unwrap();
        assert_eq!(decoded, expected);

        Ok(())
    }

    #[test]
    fn test_datetime_utc_decode() -> Result<(), BoxDynError> {
        let value =
            create_test_value_text("2020-01-02 15:30:45", DataType::Timestamp { precision: 0 });
        let decoded = <DateTime<Utc> as Decode<Odbc>>::decode(value)?;
        let expected_naive = NaiveDate::from_ymd_opt(2020, 1, 2)
            .unwrap()
            .and_hms_opt(15, 30, 45)
            .unwrap();
        let expected = DateTime::<Utc>::from_naive_utc_and_offset(expected_naive, Utc);
        assert_eq!(decoded, expected);

        Ok(())
    }

    #[test]
    fn test_naive_time_decode() -> Result<(), BoxDynError> {
        let value = create_test_value_text("15:30:45", DataType::Time { precision: 0 });
        let decoded = <NaiveTime as Decode<Odbc>>::decode(value)?;
        let expected = NaiveTime::from_hms_opt(15, 30, 45).unwrap();
        assert_eq!(decoded, expected);

        Ok(())
    }

    #[test]
    fn test_naive_date_encode() {
        let mut buf = Vec::new();
        let date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        let result = <NaiveDate as Encode<Odbc>>::encode(date, &mut buf);
        assert!(matches!(result, crate::encode::IsNull::No));
        assert_eq!(
            buf,
            vec![OdbcArgumentValue::Date(odbc_api::sys::Date {
                year: 2020,
                month: 1,
                day: 2,
            })]
        );
    }

    #[test]
    fn test_get_text_from_value() -> Result<(), BoxDynError> {
        // From text
        let value = create_test_value_text("  test  ", DataType::Varchar { length: None });
        assert_eq!(get_text_from_value(&value)?, Some("test".to_string()));

        // From empty
        let column = ColumnData {
            values: OdbcValueVec::Text(vec![String::new()]),
            type_info: OdbcTypeInfo::new(DataType::Date),
            nulls: vec![true],
        };
        let column_data = vec![Arc::new(column)];
        let batch = OdbcBatch {
            columns: Arc::new([OdbcColumn {
                name: "test".to_string(),
                type_info: OdbcTypeInfo::new(DataType::Date),
                ordinal: 0,
            }]),
            column_data,
        };
        let batch_ptr = Box::leak(Box::new(batch));
        let value = OdbcValueRef::new(batch_ptr, 0, 0);
        assert_eq!(get_text_from_value(&value)?, None);

        Ok(())
    }

    #[test]
    fn test_type_info() {
        assert_eq!(<NaiveDate as Type<Odbc>>::type_info().name(), "DATE");
        assert_eq!(<NaiveTime as Type<Odbc>>::type_info().name(), "TIME");
        assert_eq!(
            <NaiveDateTime as Type<Odbc>>::type_info().name(),
            "TIMESTAMP"
        );
        assert_eq!(
            <DateTime<Utc> as Type<Odbc>>::type_info().name(),
            "TIMESTAMP"
        );
    }
}
