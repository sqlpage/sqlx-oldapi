use crate::arguments::Arguments;
use crate::encode::Encode;
use crate::mssql::database::Mssql;
use crate::mssql::io::MssqlBufMutExt;
use crate::mssql::protocol::rpc::StatusFlags;
use crate::types::Type;
use std::fmt::{self, Write};

#[derive(Default, Clone)]
pub struct MssqlArguments {
    // next ordinal to be used when formatting a positional parameter name
    pub(crate) ordinal: usize,
    // temporary string buffer used to format parameter names
    name: String,
    pub(crate) data: Vec<u8>,
    pub(crate) declarations: String,
}

impl MssqlArguments {
    pub(crate) fn add_named<'q, T: Encode<'q, Mssql> + Type<Mssql>>(
        &mut self,
        name: &str,
        value: T,
    ) {
        let ty = value.produces().unwrap_or_else(T::type_info);

        let mut ty_name = String::new();
        ty.0.fmt(&mut ty_name);

        self.data.put_b_varchar(name); // [ParamName]
        self.data.push(0); // [StatusFlags]

        ty.0.put(&mut self.data); // [TYPE_INFO]
        ty.0.put_value(&mut self.data, value); // [ParamLenData]
    }

    pub(crate) fn add_unnamed<'q, T: Encode<'q, Mssql> + Type<Mssql>>(&mut self, value: T) {
        self.add_named("", value);
    }

    pub(crate) fn declare<'q, T: Encode<'q, Mssql> + Type<Mssql>>(
        &mut self,
        name: &str,
        initial_value: T,
    ) {
        let ty = initial_value.produces().unwrap_or_else(T::type_info);

        let mut ty_name = String::new();
        ty.0.fmt(&mut ty_name);

        self.data.put_b_varchar(name); // [ParamName]
        self.data.push(StatusFlags::BY_REF_VALUE.bits()); // [StatusFlags]

        ty.0.put(&mut self.data); // [TYPE_INFO]
        ty.0.put_value(&mut self.data, initial_value); // [ParamLenData]
    }

    pub(crate) fn append(&mut self, arguments: &mut MssqlArguments) {
        self.ordinal += arguments.ordinal;
        self.data.append(&mut arguments.data);
    }

    pub(crate) fn add<'q, T>(&mut self, value: T)
    where
        T: Encode<'q, Mssql> + Type<Mssql>,
    {
        let ty = value.produces().unwrap_or_else(T::type_info);

        // produce an ordinal parameter name
        //  @p1, @p2, ... @pN

        self.name.clear();
        self.name.push_str("@p");

        self.ordinal += 1;
        self.name.push_str(itoa::Buffer::new().format(self.ordinal));

        let MssqlArguments {
            ref name,
            ref mut declarations,
            ref mut data,
            ..
        } = self;

        // add this to our variable declaration list
        //  @p1 int, @p2 nvarchar(10), ...

        if !declarations.is_empty() {
            declarations.push(',');
        }

        declarations.push_str(name);
        declarations.push(' ');
        ty.0.fmt(declarations);

        // write out the parameter

        data.put_b_varchar(name); // [ParamName]
        data.push(0); // [StatusFlags]

        ty.0.put(data); // [TYPE_INFO]
        ty.0.put_value(data, value); // [ParamLenData]
    }
}

impl<'q> Arguments<'q> for MssqlArguments {
    type Database = Mssql;

    fn reserve(&mut self, _additional: usize, size: usize) {
        self.data.reserve(size + 10); // est. 4 chars for name, 1 for status, 1 for TYPE_INFO
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'q + Encode<'q, Self::Database> + Type<Mssql>,
    {
        self.add(value)
    }

    fn format_placeholder<W: Write>(&self, writer: &mut W) -> fmt::Result {
        // self.ordinal is incremented by the `MssqlArguments::add` method (the inherent one)
        // *before* this `format_placeholder` method is called by QueryBuilder.
        // So, `self.ordinal` correctly represents the number of the current parameter (e.g., 1 for @p1).
        writer.write_str("@p")?;
        writer.write_str(itoa::Buffer::new().format(self.ordinal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_builder::QueryBuilder;

    #[test]
    fn test_format_placeholder_method() {
        let mut args = MssqlArguments::default(); // ordinal = 0 initially
        let mut buffer = String::new();

        // Simulate first bind operation sequence as done by QueryBuilder:
        // 1. QueryBuilder calls MssqlArguments::add (via trait)
        // 2. QueryBuilder calls MssqlArguments::format_placeholder (via trait)

        // First bind:
        args.add(123i32); // This calls the inherent `MssqlArguments::add`, which increments ordinal to 1.
        args.format_placeholder(&mut buffer).unwrap(); // This should now use ordinal = 1.
        assert_eq!(buffer, "@p1");

        buffer.clear();

        // Second bind:
        args.add("test_val".to_string()); // Inherent `add` increments ordinal to 2.
        args.format_placeholder(&mut buffer).unwrap(); // This should use ordinal = 2.
        assert_eq!(buffer, "@p2");
    }

    #[test]
    fn test_query_builder_with_mssql_placeholders() {
        // This test replicates the scenario from GitHub issue #11
        let id = 100;
        let mut builder = QueryBuilder::<Mssql>::new("SELECT * FROM table ");
        builder
            .push("WHERE id=")
            .push_bind(id)
            .push(" AND name=")
            .push_bind("test");
        let sql = builder.sql(); // Get the generated SQL string

        assert_eq!(sql, "SELECT * FROM table WHERE id=@p1 AND name=@p2");
    }
}
