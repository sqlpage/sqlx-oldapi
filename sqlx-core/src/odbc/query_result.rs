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
