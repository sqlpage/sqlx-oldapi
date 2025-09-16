use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::sqlite::type_info::DataType;
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use crate::types::Type;

impl Type<Sqlite> for f32 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Float)
    }
}

impl<'q> Encode<'q, Sqlite> for f32 {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
        args.push(SqliteArgumentValue::Double((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for f32 {
    fn decode(value: SqliteValueRef<'r>) -> Result<f32, BoxDynError> {
        let dbl = value.double();
        let clamped = if dbl.is_finite() {
            dbl.clamp(f32::MIN as f64, f32::MAX as f64)
        } else {
            dbl
        };
        Ok(clamped as f32)
    }
}

impl Type<Sqlite> for f64 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Float)
    }
}

impl<'q> Encode<'q, Sqlite> for f64 {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
        args.push(SqliteArgumentValue::Double(*self));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for f64 {
    fn decode(value: SqliteValueRef<'r>) -> Result<f64, BoxDynError> {
        Ok(value.double())
    }
}
