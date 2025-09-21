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
        matches!(ty.data_type(), DataType::Date) || ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for NaiveTime {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIME
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Time { .. }) || ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for NaiveDateTime {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIMESTAMP
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Timestamp { .. })
            || ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for DateTime<Utc> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIMESTAMP
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Timestamp { .. })
            || ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for DateTime<FixedOffset> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIMESTAMP
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Timestamp { .. })
            || ty.data_type().accepts_character_data()
    }
}

impl Type<Odbc> for DateTime<Local> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::TIMESTAMP
    }
    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), DataType::Timestamp { .. })
            || ty.data_type().accepts_character_data()
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
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        Ok(s.parse()?)
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<Utc> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        Ok(s.parse()?)
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<FixedOffset> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        Ok(s.parse()?)
    }
}

impl<'r> Decode<'r, Odbc> for DateTime<Local> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Odbc>>::decode(value)?;
        Ok(s.parse::<DateTime<Utc>>()?.with_timezone(&Local))
    }
}
