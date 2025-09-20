#[derive(Debug, Default)]
pub struct OdbcQueryResult {
    pub(super) rows_affected: u64,
}

impl OdbcQueryResult {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<OdbcQueryResult> for OdbcQueryResult {
    fn extend<T: IntoIterator<Item = OdbcQueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}

#[cfg(feature = "any")]
impl From<OdbcQueryResult> for crate::any::AnyQueryResult {
    fn from(result: OdbcQueryResult) -> Self {
        crate::any::AnyQueryResult {
            rows_affected: result.rows_affected,
            last_insert_id: None, // ODBC doesn't provide last insert ID
        }
    }
}
