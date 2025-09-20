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

// Encode implementations are now in the types module

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
