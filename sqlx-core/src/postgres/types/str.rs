use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::type_info::PgType;
use crate::postgres::types::array_compatible;
use crate::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};
use crate::types::Type;
use std::borrow::Cow;

impl Type<Postgres> for str {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TEXT
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        [
            PgTypeInfo::TEXT,
            PgTypeInfo::NAME,
            PgTypeInfo::BPCHAR,
            PgTypeInfo::VARCHAR,
            PgTypeInfo::INTERVAL,
            PgTypeInfo::INT4_RANGE,
            PgTypeInfo::NUM_RANGE,
            PgTypeInfo::TS_RANGE,
            PgTypeInfo::TSTZ_RANGE,
            PgTypeInfo::DATE_RANGE,
            PgTypeInfo::INT8_RANGE,
            PgTypeInfo::MONEY,
            PgTypeInfo::UNKNOWN,
        ]
        .contains(ty)
    }
}

impl Type<Postgres> for Cow<'_, str> {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <&str as Type<Postgres>>::compatible(ty)
    }
}

impl Type<Postgres> for String {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <&str as Type<Postgres>>::compatible(ty)
    }
}

impl PgHasArrayType for &'_ str {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TEXT_ARRAY
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        array_compatible::<&str>(ty)
    }
}

impl PgHasArrayType for Cow<'_, str> {
    fn array_type_info() -> PgTypeInfo {
        <&str as PgHasArrayType>::array_type_info()
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        <&str as PgHasArrayType>::array_compatible(ty)
    }
}

impl PgHasArrayType for String {
    fn array_type_info() -> PgTypeInfo {
        <&str as PgHasArrayType>::array_type_info()
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        <&str as PgHasArrayType>::array_compatible(ty)
    }
}

impl Encode<'_, Postgres> for &'_ str {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(self.as_bytes());

        IsNull::No
    }
}

impl Encode<'_, Postgres> for Cow<'_, str> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        match self {
            Cow::Borrowed(str) => <&str as Encode<Postgres>>::encode(*str, buf),
            Cow::Owned(str) => <&str as Encode<Postgres>>::encode(&**str, buf),
        }
    }
}

impl Encode<'_, Postgres> for String {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        <&str as Encode<Postgres>>::encode(&**self, buf)
    }
}

impl<'r> Decode<'r, Postgres> for &'r str {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        value.as_str()
    }
}

impl<'r> Decode<'r, Postgres> for Cow<'r, str> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(Cow::Borrowed(value.as_str()?))
    }
}

impl Decode<'_, Postgres> for String {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match *value.type_info {
            PgType::Interval => super::interval::decode_as_string(value),
            PgType::Int4Range => super::range::decode_as_string::<i32>(value),
            #[cfg(feature = "bigdecimal")]
            PgType::NumRange => super::range::decode_as_string::<bigdecimal::BigDecimal>(value),
            #[cfg(feature = "chrono")]
            PgType::TsRange => super::range::decode_as_string::<chrono::NaiveDateTime>(value),
            #[cfg(feature = "chrono")]
            PgType::TstzRange => {
                super::range::decode_as_string::<chrono::DateTime<chrono::Utc>>(value)
            }
            #[cfg(feature = "chrono")]
            PgType::DateRange => super::range::decode_as_string::<chrono::NaiveDate>(value),
            PgType::Int8Range => super::range::decode_as_string::<i64>(value),
            _ => Ok(value.as_str()?.to_owned()),
        }
    }
}
