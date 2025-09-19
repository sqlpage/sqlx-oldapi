use crate::arguments::Arguments;
use crate::encode::Encode;
use crate::odbc::Odbc;
use crate::types::Type;

#[derive(Default)]
pub struct OdbcArguments<'q> {
    pub(crate) values: Vec<OdbcArgumentValue<'q>>,
}

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
