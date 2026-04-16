use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;
use chrono::{
    DateTime, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone,
    Utc,
};
use std::mem;

impl Type<Postgres> for NaiveDateTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP
    }
}

impl<Tz: TimeZone> Type<Postgres> for DateTime<Tz> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ
    }
}

impl PgHasArrayType for NaiveDateTime {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP_ARRAY
    }
}

impl<Tz: TimeZone> PgHasArrayType for DateTime<Tz> {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ_ARRAY
    }
}

impl Encode<'_, Postgres> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // FIXME: We should *really* be returning an error, Encode needs to be fallible
        // TIMESTAMP is encoded as the microseconds since the epoch
        let epoch = NaiveDate::from_ymd_opt(2000, 1, 1)
            .unwrap()
            .and_time(NaiveTime::default());
        let us = (*self - epoch)
            .num_microseconds()
            .unwrap_or_else(|| panic!("NaiveDateTime out of range for Postgres: {:?}", self));

        Encode::<Postgres>::encode(us, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for NaiveDateTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIMESTAMP is encoded as the microseconds since the epoch
                let epoch = NaiveDate::from_ymd_opt(2000, 1, 1)
                    .unwrap()
                    .and_time(NaiveTime::default());
                let us = Decode::<Postgres>::decode(value)?;
                epoch + Duration::microseconds(us)
            }

            PgValueFormat::Text => {
                let s = value.as_str()?;
                // Try with timezone offset first (handles both positive and
                // negative offsets like +05 or -05), then fall back to parsing
                // without timezone for plain timestamps.
                NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f%#z")
                    .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f"))?
            }
        })
    }
}

impl<Tz: TimeZone> Encode<'_, Postgres> for DateTime<Tz> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        Encode::<Postgres>::encode(self.naive_utc(), buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for DateTime<Local> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let naive = <NaiveDateTime as Decode<Postgres>>::decode(value)?;
        Ok(Local.from_utc_datetime(&naive))
    }
}

impl<'r> Decode<'r, Postgres> for DateTime<Utc> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let naive = <NaiveDateTime as Decode<Postgres>>::decode(value)?;
        Ok(Utc.from_utc_datetime(&naive))
    }
}

impl<'r> Decode<'r, Postgres> for DateTime<FixedOffset> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let naive = <NaiveDateTime as Decode<Postgres>>::decode(value)?;
        Ok(Utc.fix().from_utc_datetime(&naive))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::PgValueFormat;

    fn text_decode_naive(s: &str) -> Result<NaiveDateTime, BoxDynError> {
        let value = PgValueRef {
            value: Some(s.as_bytes()),
            row: None,
            type_info: PgTypeInfo::TIMESTAMPTZ,
            format: PgValueFormat::Text,
        };
        <NaiveDateTime as Decode<Postgres>>::decode(value)
    }

    #[test]
    fn test_decode_timestamptz_negative_offset() {
        // PostgreSQL returns this text format when session timezone is America/New_York (UTC-5)
        let dt = text_decode_naive("2020-12-31 19:00:00-05").unwrap();
        assert_eq!(
            dt,
            NaiveDate::from_ymd_opt(2020, 12, 31)
                .unwrap()
                .and_hms_opt(19, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_decode_timestamptz_positive_offset() {
        let dt = text_decode_naive("2021-01-01 05:00:00+05").unwrap();
        assert_eq!(
            dt,
            NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(5, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_decode_timestamp_no_offset() {
        let dt = text_decode_naive("2021-01-01 00:00:00").unwrap();
        assert_eq!(
            dt,
            NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_decode_timestamptz_with_fractional_seconds() {
        let dt = text_decode_naive("2021-01-01 00:00:00.123456-05").unwrap();
        assert_eq!(
            dt,
            NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_micro_opt(0, 0, 0, 123456)
                .unwrap()
        );
    }
}
