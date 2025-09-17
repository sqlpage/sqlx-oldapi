use std::mem;

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;

// `PgInterval` is available for direct access to the INTERVAL type

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PgInterval {
    pub months: i32,
    pub days: i32,
    pub microseconds: i64,
}

/// Decode an interval value as a string representation
pub fn decode_as_string(value: PgValueRef<'_>) -> Result<String, BoxDynError> {
    match value.format() {
        PgValueFormat::Binary => {
            let interval = PgInterval::decode(value)?;
            Ok(interval.to_string())
        }

        PgValueFormat::Text => {
            // For text format, we can just return the string as-is
            Ok(value.as_str()?.to_owned())
        }
    }
}

impl Type<Postgres> for PgInterval {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL
    }
}

impl PgHasArrayType for PgInterval {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL_ARRAY
    }
}

impl<'de> Decode<'de, Postgres> for PgInterval {
    fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let mut buf = value.as_bytes()?;
                let microseconds = buf.read_i64::<NetworkEndian>()?;
                let days = buf.read_i32::<NetworkEndian>()?;
                let months = buf.read_i32::<NetworkEndian>()?;

                Ok(PgInterval {
                    months,
                    days,
                    microseconds,
                })
            }

            // TODO: Implement parsing of text mode
            PgValueFormat::Text => {
                Err("not implemented: decode `INTERVAL` in text mode (unprepared queries)".into())
            }
        }
    }
}

impl Encode<'_, Postgres> for PgInterval {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.microseconds.to_be_bytes());
        buf.extend(&self.days.to_be_bytes());
        buf.extend(&self.months.to_be_bytes());

        IsNull::No
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

impl std::fmt::Display for PgInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();

        if self.months != 0 {
            let years = self.months / 12;
            let months = self.months % 12;

            if years != 0 {
                parts.push(format!(
                    "{} year{}",
                    years,
                    if years.abs() != 1 { "s" } else { "" }
                ));
            }
            if months != 0 {
                parts.push(format!(
                    "{} mon{}",
                    months,
                    if months.abs() != 1 { "s" } else { "" }
                ));
            }
        }

        if self.days != 0 {
            parts.push(format!(
                "{} day{}",
                self.days,
                if self.days.abs() != 1 { "s" } else { "" }
            ));
        }

        let time_us = self.microseconds;

        if time_us != 0 || parts.is_empty() {
            let sign = if time_us < 0 { "-" } else { "" };
            let us = time_us.abs();

            let total_seconds = us / 1_000_000;
            let microseconds = us % 1_000_000;
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;

            let time_part = if microseconds != 0 {
                format!(
                    "{}{:02}:{:02}:{:02}.{:06}",
                    sign, hours, minutes, seconds, microseconds
                )
            } else {
                format!("{}{:02}:{:02}:{:02}", sign, hours, minutes, seconds)
            };
            parts.push(time_part);
        }

        if parts.is_empty() {
            write!(f, "00:00:00")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

// We then implement Encode + Type for std Duration, chrono Duration, and time Duration
// This is to enable ease-of-use for encoding when its simple

impl Type<Postgres> for std::time::Duration {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL
    }
}

impl PgHasArrayType for std::time::Duration {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL_ARRAY
    }
}

impl Encode<'_, Postgres> for std::time::Duration {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        PgInterval::try_from(*self)
            .expect("failed to encode `std::time::Duration`")
            .encode_by_ref(buf)
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

impl TryFrom<std::time::Duration> for PgInterval {
    type Error = BoxDynError;

    /// Convert a `std::time::Duration` to a `PgInterval`
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// microsecond overflow.
    fn try_from(value: std::time::Duration) -> Result<Self, BoxDynError> {
        if !value.as_nanos().is_multiple_of(1000) {
            return Err("PostgreSQL `INTERVAL` does not support nanoseconds precision".into());
        }

        Ok(Self {
            months: 0,
            days: 0,
            microseconds: value.as_micros().try_into()?,
        })
    }
}

#[cfg(feature = "chrono")]
impl Type<Postgres> for chrono::Duration {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL
    }
}

#[cfg(feature = "chrono")]
impl PgHasArrayType for chrono::Duration {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL_ARRAY
    }
}

#[cfg(feature = "chrono")]
impl Encode<'_, Postgres> for chrono::Duration {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let pg_interval = PgInterval::try_from(*self).expect("Failed to encode chrono::Duration");
        pg_interval.encode_by_ref(buf)
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<chrono::Duration> for PgInterval {
    type Error = BoxDynError;

    /// Convert a `chrono::Duration` to a `PgInterval`.
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// nanosecond overflow.
    fn try_from(value: chrono::Duration) -> Result<Self, BoxDynError> {
        value
            .num_nanoseconds()
            .map_or::<Result<_, Self::Error>, _>(
                Err("Overflow has occurred for PostgreSQL `INTERVAL`".into()),
                |nanoseconds| {
                    if nanoseconds % 1000 != 0 {
                        return Err(
                            "PostgreSQL `INTERVAL` does not support nanoseconds precision".into(),
                        );
                    }
                    Ok(())
                },
            )?;

        value.num_microseconds().map_or(
            Err("Overflow has occurred for PostgreSQL `INTERVAL`".into()),
            |microseconds| {
                Ok(Self {
                    months: 0,
                    days: 0,
                    microseconds,
                })
            },
        )
    }
}

#[cfg(feature = "time")]
impl Type<Postgres> for time::Duration {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL
    }
}

#[cfg(feature = "time")]
impl PgHasArrayType for time::Duration {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL_ARRAY
    }
}

#[cfg(feature = "time")]
impl Encode<'_, Postgres> for time::Duration {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let pg_interval = PgInterval::try_from(*self).expect("Failed to encode time::Duration");
        pg_interval.encode_by_ref(buf)
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

#[cfg(feature = "time")]
impl TryFrom<time::Duration> for PgInterval {
    type Error = BoxDynError;

    /// Convert a `time::Duration` to a `PgInterval`.
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// microsecond overflow.
    fn try_from(value: time::Duration) -> Result<Self, BoxDynError> {
        if value.whole_nanoseconds() % 1000 != 0 {
            return Err("PostgreSQL `INTERVAL` does not support nanoseconds precision".into());
        }

        Ok(Self {
            months: 0,
            days: 0,
            microseconds: value.whole_microseconds().try_into()?,
        })
    }
}

#[test]
fn test_encode_interval() {
    let mut buf = PgArgumentBuffer::default();

    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 0,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    buf.clear();

    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 1_000,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 0, 3, 232, 0, 0, 0, 0, 0, 0, 0, 0]);
    buf.clear();

    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 1_000_000,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 15, 66, 64, 0, 0, 0, 0, 0, 0, 0, 0]);
    buf.clear();

    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 3_600_000_000,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(
        &**buf,
        [0, 0, 0, 0, 214, 147, 164, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    buf.clear();

    let interval = PgInterval {
        months: 0,
        days: 1,
        microseconds: 0,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
    buf.clear();

    let interval = PgInterval {
        months: 1,
        days: 0,
        microseconds: 0,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    buf.clear();
}

#[test]
fn test_pginterval_std() {
    // Case for positive duration
    let interval = PgInterval {
        days: 0,
        months: 0,
        microseconds: 27_000,
    };
    assert_eq!(
        &PgInterval::try_from(std::time::Duration::from_micros(27_000)).unwrap(),
        &interval
    );

    // Case when precision loss occurs
    assert!(PgInterval::try_from(std::time::Duration::from_nanos(27_000_001)).is_err());

    // Case when microsecond overflow occurs
    assert!(PgInterval::try_from(std::time::Duration::from_secs(20_000_000_000_000)).is_err());
}

#[test]
#[cfg(feature = "chrono")]
fn test_pginterval_chrono() {
    // Case for positive duration
    let interval = PgInterval {
        days: 0,
        months: 0,
        microseconds: 27_000,
    };
    assert_eq!(
        &PgInterval::try_from(chrono::Duration::microseconds(27_000)).unwrap(),
        &interval
    );

    // Case for negative duration
    let interval = PgInterval {
        days: 0,
        months: 0,
        microseconds: -27_000,
    };
    assert_eq!(
        &PgInterval::try_from(chrono::Duration::microseconds(-27_000)).unwrap(),
        &interval
    );

    // Case when precision loss occurs
    assert!(PgInterval::try_from(chrono::Duration::nanoseconds(27_000_001)).is_err());
    assert!(PgInterval::try_from(chrono::Duration::nanoseconds(-27_000_001)).is_err());

    // Case when nanosecond overflow occurs
    assert!(PgInterval::try_from(chrono::Duration::seconds(10_000_000_000)).is_err());
    assert!(PgInterval::try_from(chrono::Duration::seconds(-10_000_000_000)).is_err());
}

#[test]
#[cfg(feature = "time")]
fn test_pginterval_time() {
    // Case for positive duration
    let interval = PgInterval {
        days: 0,
        months: 0,
        microseconds: 27_000,
    };
    assert_eq!(
        &PgInterval::try_from(time::Duration::microseconds(27_000)).unwrap(),
        &interval
    );

    // Case for negative duration
    let interval = PgInterval {
        days: 0,
        months: 0,
        microseconds: -27_000,
    };
    assert_eq!(
        &PgInterval::try_from(time::Duration::microseconds(-27_000)).unwrap(),
        &interval
    );

    // Case when precision loss occurs
    assert!(PgInterval::try_from(time::Duration::nanoseconds(27_000_001)).is_err());
    assert!(PgInterval::try_from(time::Duration::nanoseconds(-27_000_001)).is_err());

    // Case when microsecond overflow occurs
    assert!(PgInterval::try_from(time::Duration::seconds(10_000_000_000_000)).is_err());
    assert!(PgInterval::try_from(time::Duration::seconds(-10_000_000_000_000)).is_err());
}

#[test]
fn test_pginterval_display() {
    // Zero interval
    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 0,
    };
    assert_eq!(interval.to_string(), "00:00:00");

    // Time only
    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 3_600_000_000, // 1 hour
    };
    assert_eq!(interval.to_string(), "01:00:00");

    // Time with microseconds
    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 3_660_000_000, // 1 hour 1 minute
    };
    assert_eq!(interval.to_string(), "01:01:00");

    // Time with microseconds
    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 3_600 * 1_000_000 + 60 * 1_000_000 + 30, // 1 hour 1 minute 30 microseconds
    };
    assert_eq!(interval.to_string(), "01:01:00.000030");

    // Days only
    let interval = PgInterval {
        months: 0,
        days: 27,
        microseconds: 0,
    };
    assert_eq!(interval.to_string(), "27 days");

    // Months only
    let interval = PgInterval {
        months: 11,
        days: 0,
        microseconds: 0,
    };
    assert_eq!(interval.to_string(), "11 mons");

    // Years and months
    let interval = PgInterval {
        months: 14, // 1 year 2 months
        days: 0,
        microseconds: 0,
    };
    assert_eq!(interval.to_string(), "1 year 2 mons");

    // Complex interval
    let interval = PgInterval {
        months: 14, // 1 year 2 months
        days: 27,
        microseconds: 43_200_000_000 + 180_000_000 + 30_000, // 12 hours 3 minutes 30 milliseconds
    };
    assert_eq!(
        interval.to_string(),
        "1 year 2 mons 27 days 12:03:00.030000"
    );

    // Negative microseconds
    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: -1_000_000, // -1 second
    };
    assert_eq!(interval.to_string(), "-00:00:01");
}
