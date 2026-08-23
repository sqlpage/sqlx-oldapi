use crate::arguments::{Arguments, NamedArguments};
use crate::encode::{Encode, IsNull};
use crate::error::Error;
use crate::sqlite::statement::StatementHandle;
use crate::sqlite::Sqlite;
use atoi::atoi;
use libsqlite3_sys::SQLITE_OK;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum SqliteArgumentValue<'q> {
    Null,
    Text(Cow<'q, str>),
    Blob(Cow<'q, [u8]>),
    Double(f64),
    Int(i32),
    Int64(i64),
}

#[derive(Default, Debug, Clone)]
pub struct SqliteArguments<'q> {
    pub(crate) values: Vec<SqliteArgumentValue<'q>>,
    // (index into values, exact SQLite parameter token)
    pub(crate) named: Vec<(usize, Cow<'q, str>)>,
}

impl<'q> SqliteArguments<'q> {
    pub(crate) fn add<T>(&mut self, value: T)
    where
        T: Encode<'q, Sqlite>,
    {
        if let IsNull::Yes = value.encode(&mut self.values) {
            self.values.push(SqliteArgumentValue::Null);
        }
    }

    pub(crate) fn into_static(self) -> SqliteArguments<'static> {
        SqliteArguments {
            values: self
                .values
                .into_iter()
                .map(SqliteArgumentValue::into_static)
                .collect(),
            named: self
                .named
                .into_iter()
                .map(|(index, name)| (index, Cow::Owned(name.into_owned())))
                .collect(),
        }
    }
}

impl<'q> Arguments<'q> for SqliteArguments<'q> {
    type Database = Sqlite;

    fn reserve(&mut self, len: usize, _size_hint: usize) {
        self.values.reserve(len);
    }

    fn add<T>(&mut self, value: T)
    where
        T: Encode<'q, Self::Database>,
    {
        self.add(value)
    }
}

impl<'q> NamedArguments<'q> for SqliteArguments<'q> {
    fn add_named<T>(&mut self, name: &'q str, value: T)
    where
        T: 'q + Send + Encode<'q, Self::Database> + crate::types::Type<Self::Database>,
    {
        self.add(value);
        self.named
            .push((self.values.len() - 1, Cow::Borrowed(name)));
    }
}

impl SqliteArguments<'_> {
    pub(super) fn bind(&self, handle: &mut StatementHandle, offset: usize) -> Result<usize, Error> {
        let mut arg_i = offset;

        if !self.named.is_empty() && self.named.len() != self.values.len() {
            return Err(err_protocol!(
                "cannot mix named and positional SQLite parameters"
            ));
        }

        let cnt = handle.bind_parameter_count();

        // SQLite resolves exact parameter tokens natively. Keep only the name metadata here;
        // no Rust hashmap or owned NUL-terminated name is needed.
        for (value_i, name) in &self.named {
            let param_i = handle
                .bind_parameter_index(name)
                .ok_or_else(|| err_protocol!("unknown SQLite parameter: {}", name))?;

            self.values[*value_i].bind(handle, param_i)?;
        }

        for param_i in 1..=cnt {
            let parameter_name = handle.bind_parameter_name(param_i);

            // figure out the index of this bind parameter into our argument tuple
            let n: usize = if let Some(name) = parameter_name {
                if self
                    .named
                    .iter()
                    .any(|(_, bound_name)| bound_name.as_ref() == name)
                {
                    continue;
                } else if let Some(name) = name.strip_prefix('?') {
                    // parameter should have the form ?NNN
                    atoi(name.as_bytes()).expect("parameter of the form ?NNN")
                } else if let Some(name) = name.strip_prefix('$') {
                    // parameter should have the form $NNN
                    atoi(name.as_bytes()).ok_or_else(|| {
                        err_protocol!("named SQLite parameter was not bound: {}", name)
                    })?
                } else {
                    return Err(err_protocol!(
                        "named SQLite parameter was not bound: {}",
                        name
                    ));
                }
            } else {
                arg_i += 1;
                arg_i
            };

            if n > self.values.len() {
                // SQLite treats unbound variables as NULL
                // we reproduce this here
                // If you are reading this and think this should be an error, open an issue and we can
                // discuss configuring this somehow
                // Note that the query macros have a different way of enforcing
                // argument arity
                break;
            }

            self.values[n - 1].bind(handle, param_i)?;
        }

        Ok(arg_i - offset)
    }
}

impl SqliteArgumentValue<'_> {
    fn into_static(self) -> SqliteArgumentValue<'static> {
        use SqliteArgumentValue::*;

        match self {
            Null => Null,
            Text(text) => Text(text.into_owned().into()),
            Blob(blob) => Blob(blob.into_owned().into()),
            Int(v) => Int(v),
            Int64(v) => Int64(v),
            Double(v) => Double(v),
        }
    }

    fn bind(&self, handle: &mut StatementHandle, i: usize) -> Result<(), Error> {
        use SqliteArgumentValue::*;

        let status = match self {
            Text(v) => handle.bind_text(i, v),
            Blob(v) => handle.bind_blob(i, v),
            Int(v) => handle.bind_int(i, *v),
            Int64(v) => handle.bind_int64(i, *v),
            Double(v) => handle.bind_double(i, *v),
            Null => handle.bind_null(i),
        };

        if status != SQLITE_OK {
            return Err(handle.last_error().into());
        }

        Ok(())
    }
}
