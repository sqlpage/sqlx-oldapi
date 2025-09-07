use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;
use uuid::Uuid;
use std::convert::TryInto;

impl Type<Mssql> for Uuid {
    fn type_info() -> MssqlTypeInfo {
        // MSSQL's UNIQUEIDENTIFIER type
        MssqlTypeInfo(TypeInfo {
            ty: DataType::Guid,
            size: 16,
            scale: 0,
            precision: 0,
            collation: None,
        })
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Guid)
    }
}

impl Encode<'_, Mssql> for Uuid {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_bytes_le());
        IsNull::No
    }
}

impl Decode<'_, Mssql> for Uuid {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes: [u8; 16] = value.as_bytes()?.try_into()?;
        Ok(Uuid::from_bytes_le(bytes))
    }
}
