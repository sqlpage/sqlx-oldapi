use crate::error::DatabaseError;
use odbc_api::{
    handles::{slice_to_cow_utf8, Record},
    Error as OdbcApiError,
};
use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub struct OdbcDatabaseError {
    error: OdbcApiError,
    message: String,
    code: Option<String>,
}

impl OdbcDatabaseError {
    fn diagnostic_record(error: &OdbcApiError) -> Option<&Record> {
        match error {
            OdbcApiError::Diagnostics { record, .. } => Some(record),
            OdbcApiError::InvalidRowArraySize { record, .. } => Some(record),
            OdbcApiError::UnsupportedOdbcApiVersion(record) => Some(record),
            OdbcApiError::UnableToRepresentNull(record) => Some(record),
            OdbcApiError::OracleOdbcDriverDoesNotSupport64Bit(record) => Some(record),
            _ => None,
        }
    }

    fn diagnostic_code(record: &Record) -> Option<String> {
        let code = record.state.as_str();

        if code.as_bytes().iter().all(|&byte| byte == 0) {
            None
        } else {
            Some(code.to_owned())
        }
    }
}

impl From<OdbcApiError> for OdbcDatabaseError {
    fn from(error: OdbcApiError) -> Self {
        let record = Self::diagnostic_record(&error);
        let message = record
            .map(|record| slice_to_cow_utf8(&record.message).into_owned())
            .filter(|message| !message.is_empty())
            .unwrap_or_else(|| error.to_string());
        let code = record.and_then(Self::diagnostic_code);

        Self {
            error,
            message,
            code,
        }
    }
}

impl Display for OdbcDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.error, f)
    }
}

impl std::error::Error for OdbcDatabaseError {}

impl DatabaseError for OdbcDatabaseError {
    fn message(&self) -> &str {
        &self.message
    }
    fn code(&self) -> Option<Cow<'_, str>> {
        self.code.as_deref().map(Cow::Borrowed)
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
        crate::error::Error::Database(Box::new(OdbcDatabaseError::from(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DatabaseError;
    use odbc_api::handles::{Record, SqlChar, State};

    fn sql_chars(text: &str) -> Vec<SqlChar> {
        text.bytes().map(Into::into).collect()
    }

    #[test]
    fn database_error_uses_odbc_diagnostics_for_message_and_code() {
        let error = OdbcDatabaseError::from(OdbcApiError::Diagnostics {
            function: "SQLExecDirect",
            record: Record {
                state: State(*b"HY000"),
                native_error: 1234,
                message: sql_chars("syntax error near FROM"),
            },
        });

        assert_eq!(error.message(), "syntax error near FROM");
        assert_eq!(error.code().as_deref(), Some("HY000"));
    }
}
