use std::collections::HashMap;

/// The result of a query to a Snowflake database.
#[derive(Debug, Default)]
pub struct SnowflakeQueryResult {
    pub(crate) rows_affected: u64,
    pub(crate) last_insert_id: Option<i64>,
}

impl SnowflakeQueryResult {
    pub(crate) fn new(rows_affected: u64, last_insert_id: Option<i64>) -> Self {
        Self {
            rows_affected,
            last_insert_id,
        }
    }

    /// Returns the number of rows affected by the query.
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }

    /// Returns the last insert ID, if available.
    pub fn last_insert_id(&self) -> Option<i64> {
        self.last_insert_id
    }
}

impl Extend<SnowflakeQueryResult> for SnowflakeQueryResult {
    fn extend<T: IntoIterator<Item = SnowflakeQueryResult>>(&mut self, iter: T) {
        for result in iter {
            self.rows_affected += result.rows_affected;
            // Keep the last insert ID from the most recent result
            if result.last_insert_id.is_some() {
                self.last_insert_id = result.last_insert_id;
            }
        }
    }
}