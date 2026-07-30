use crate::column::Column;
use crate::odbc::{Odbc, OdbcTypeInfo};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct OdbcColumn {
    pub(crate) name: String,
    pub(crate) type_info: OdbcTypeInfo,
    pub(crate) ordinal: usize,
}

impl Column for OdbcColumn {
    type Database = Odbc;

    fn ordinal(&self) -> usize {
        self.ordinal
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn type_info(&self) -> &OdbcTypeInfo {
        &self.type_info
    }
}

#[cfg(feature = "any")]
impl From<OdbcColumn> for crate::any::AnyColumn {
    fn from(col: OdbcColumn) -> Self {
        crate::any::AnyColumn {
            kind: crate::any::column::AnyColumnKind::Odbc(col.clone()),
            type_info: crate::any::AnyTypeInfo::from(col.type_info),
        }
    }
}

mod private {
    use super::OdbcColumn;
    use crate::column::private_column::Sealed;
    impl Sealed for OdbcColumn {}
}

#[cfg(all(test, feature = "offline", feature = "json"))]
mod offline_tests {
    use super::*;
    use crate::describe::Describe;
    use either::Either;

    #[test]
    fn describe_round_trips_for_offline_queries() {
        let describe = Describe::<Odbc> {
            columns: vec![OdbcColumn {
                name: "value".to_owned(),
                type_info: OdbcTypeInfo::INTEGER,
                ordinal: 0,
            }],
            parameters: Some(Either::Left(vec![OdbcTypeInfo::INTEGER])),
            nullable: vec![Some(false)],
        };

        let json = serde_json::to_string(&describe).unwrap();
        let deserialized: Describe<Odbc> = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.columns().len(), 1);
        assert_eq!(deserialized.columns()[0].name(), "value");
        assert_eq!(
            deserialized.columns()[0].type_info(),
            &OdbcTypeInfo::INTEGER
        );
        assert_eq!(
            deserialized.parameters(),
            Some(Either::Left(&[OdbcTypeInfo::INTEGER][..]))
        );
        assert_eq!(deserialized.nullable(0), Some(false));
    }
}
