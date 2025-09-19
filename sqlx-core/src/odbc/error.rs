use crate::error::DatabaseError;
use odbc_api::Error as OdbcApiError;
use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub struct OdbcDatabaseError(pub OdbcApiError);

impl Display for OdbcDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl std::error::Error for OdbcDatabaseError {}

impl DatabaseError for OdbcDatabaseError {
    fn message(&self) -> &str {
        "ODBC error"
    }
    fn code(&self) -> Option<Cow<'_, str>> {
        None
    }
    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
}

impl From<OdbcApiError> for crate::error::Error {
    fn from(value: OdbcApiError) -> Self {
        crate::error::Error::Database(Box::new(OdbcDatabaseError(value)))
    }
}
