use crate::error::DatabaseError;
use std::borrow::Cow;
use std::fmt::{self, Display};

/// An error returned by the Snowflake database.
#[derive(Debug)]
pub struct SnowflakeDatabaseError {
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) sql_state: Option<String>,
}

impl SnowflakeDatabaseError {
    pub fn new(code: String, message: String, sql_state: Option<String>) -> Self {
        Self {
            code,
            message,
            sql_state,
        }
    }
}

impl Display for SnowflakeDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SnowflakeDatabaseError {}

impl DatabaseError for SnowflakeDatabaseError {
    fn message(&self) -> &str {
        &self.message
    }

    fn constraint(&self) -> Option<&str> {
        None
    }

    fn code(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.code))
    }

    #[doc(hidden)]
    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
}
