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

    fn add<T>(&mut self, _value: T)
    where
        T: 'q + Send + Encode<'q, Self::Database> + Type<Self::Database>,
    {
        // Not implemented yet; ODBC backend currently executes direct SQL without binds
        // This stub allows query() without binds to compile.
        let _ = _value;
    }
}
