use crate::arguments::Arguments;
use crate::encode::Encode;
use crate::odbc::Odbc;
use crate::types::Type;

#[derive(Default)]
pub struct OdbcArguments<'q> {
    pub(crate) values: Vec<OdbcArgumentValue<'q>>,
}

#[derive(Debug, Clone)]
pub enum OdbcArgumentValue<'q> {
    Text(String),
    Bytes(Vec<u8>),
    Int(i64),
    Float(f64),
    Null,
    // Borrowed placeholder to satisfy lifetimes; not used for now
    Phantom(std::marker::PhantomData<&'q ()>),
}

impl<'q> Arguments<'q> for OdbcArguments<'q> {
    type Database = Odbc;

    fn reserve(&mut self, additional: usize, _size: usize) {
        self.values.reserve(additional);
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'q + Send + Encode<'q, Self::Database> + Type<Self::Database>,
    {
        let _ = value.encode(&mut self.values);
    }
}

impl<'q> Encode<'q, Odbc> for i32 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i64 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for f32 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Float(self as f64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Float(*self as f64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for f64 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Float(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Float(*self));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for String {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.clone()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for &'q str {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text(self.to_owned()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Text((*self).to_owned()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for Vec<u8> {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.clone()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for &'q [u8] {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.to_vec()));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Bytes(self.to_vec()));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i16 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for i8 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u8 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u16 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u32 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(self as i64));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(*self as i64));
        crate::encode::IsNull::No
    }
}

impl<'q> Encode<'q, Odbc> for u64 {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        match i64::try_from(self) {
            Ok(value) => {
                buf.push(OdbcArgumentValue::Int(value));
                crate::encode::IsNull::No
            }
            Err(_) => {
                log::warn!("u64 value {} too large for ODBC, encoding as NULL", self);
                buf.push(OdbcArgumentValue::Null);
                crate::encode::IsNull::Yes
            }
        }
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        match i64::try_from(*self) {
            Ok(value) => {
                buf.push(OdbcArgumentValue::Int(value));
                crate::encode::IsNull::No
            }
            Err(_) => {
                log::warn!("u64 value {} too large for ODBC, encoding as NULL", self);
                buf.push(OdbcArgumentValue::Null);
                crate::encode::IsNull::Yes
            }
        }
    }
}

impl<'q> Encode<'q, Odbc> for bool {
    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(if self { 1 } else { 0 }));
        crate::encode::IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        buf.push(OdbcArgumentValue::Int(if *self { 1 } else { 0 }));
        crate::encode::IsNull::No
    }
}

// Feature-gated Encode implementations
#[cfg(feature = "chrono")]
mod chrono_encode {
    use super::*;
    use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};

    impl<'q> Encode<'q, Odbc> for NaiveDate {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.format("%Y-%m-%d").to_string()));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.format("%Y-%m-%d").to_string()));
            crate::encode::IsNull::No
        }
    }

    impl<'q> Encode<'q, Odbc> for NaiveTime {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.format("%H:%M:%S").to_string()));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.format("%H:%M:%S").to_string()));
            crate::encode::IsNull::No
        }
    }

    impl<'q> Encode<'q, Odbc> for NaiveDateTime {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(
                self.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(
                self.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
            crate::encode::IsNull::No
        }
    }

    impl<'q> Encode<'q, Odbc> for DateTime<Utc> {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(
                self.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(
                self.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
            crate::encode::IsNull::No
        }
    }

    impl<'q> Encode<'q, Odbc> for DateTime<FixedOffset> {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(
                self.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(
                self.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
            crate::encode::IsNull::No
        }
    }

    impl<'q> Encode<'q, Odbc> for DateTime<Local> {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(
                self.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(
                self.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
            crate::encode::IsNull::No
        }
    }
}

#[cfg(feature = "json")]
mod json_encode {
    use super::*;
    use serde_json::Value;

    impl<'q> Encode<'q, Odbc> for Value {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.to_string()));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.to_string()));
            crate::encode::IsNull::No
        }
    }
}

#[cfg(feature = "bigdecimal")]
mod bigdecimal_encode {
    use super::*;
    use bigdecimal::BigDecimal;

    impl<'q> Encode<'q, Odbc> for BigDecimal {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.to_string()));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.to_string()));
            crate::encode::IsNull::No
        }
    }
}

#[cfg(feature = "decimal")]
mod decimal_encode {
    use super::*;
    use rust_decimal::Decimal;

    impl<'q> Encode<'q, Odbc> for Decimal {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.to_string()));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.to_string()));
            crate::encode::IsNull::No
        }
    }
}

#[cfg(feature = "uuid")]
mod uuid_encode {
    use super::*;
    use uuid::Uuid;

    impl<'q> Encode<'q, Odbc> for Uuid {
        fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.to_string()));
            crate::encode::IsNull::No
        }

        fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
            buf.push(OdbcArgumentValue::Text(self.to_string()));
            crate::encode::IsNull::No
        }
    }
}

impl<'q, T> Encode<'q, Odbc> for Option<T>
where
    T: Encode<'q, Odbc> + Type<Odbc> + 'q,
{
    fn produces(&self) -> Option<crate::odbc::OdbcTypeInfo> {
        if let Some(v) = self {
            v.produces()
        } else {
            T::type_info().into()
        }
    }

    fn encode(self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        match self {
            Some(v) => v.encode(buf),
            None => {
                buf.push(OdbcArgumentValue::Null);
                crate::encode::IsNull::Yes
            }
        }
    }

    fn encode_by_ref(&self, buf: &mut Vec<OdbcArgumentValue<'q>>) -> crate::encode::IsNull {
        match self {
            Some(v) => v.encode_by_ref(buf),
            None => {
                buf.push(OdbcArgumentValue::Null);
                crate::encode::IsNull::Yes
            }
        }
    }

    fn size_hint(&self) -> usize {
        self.as_ref().map_or(0, Encode::size_hint)
    }
}
