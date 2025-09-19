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
    pub(crate) fn new(code: String, message: String, sql_state: Option<String>) -> Self {
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

    fn details(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn hint(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn table_name(&self) -> Option<&str> {
        None
    }

    fn column_name(&self) -> Option<&str> {
        None
    }

    fn constraint_name(&self) -> Option<&str> {
        None
    }

    fn kind(&self) -> crate::error::ErrorKind {
        // Map Snowflake error codes to SQLx error kinds
        match self.code.as_str() {
            // Authentication/authorization errors
            "390100" | "390101" | "390102" | "390103" | "390104" | "390105" | "390106" | "390107" 
            | "390108" | "390109" | "390110" | "390111" | "390112" | "390113" | "390114" | "390115" 
            | "390116" | "390117" | "390118" | "390119" | "390120" | "390121" | "390122" | "390123" 
            | "390124" | "390125" | "390126" | "390127" | "390128" | "390129" | "390130" | "390131" 
            | "390132" | "390133" | "390134" | "390135" | "390136" | "390137" | "390138" | "390139" 
            | "390140" | "390141" | "390142" | "390143" | "390144" | "390145" | "390146" | "390147" 
            | "390148" | "390149" | "390150" | "390151" | "390152" | "390153" | "390154" | "390155" 
            | "390156" | "390157" | "390158" | "390159" | "390160" | "390161" | "390162" | "390163" 
            | "390164" | "390165" | "390166" | "390167" | "390168" | "390169" | "390170" | "390171" 
            | "390172" | "390173" | "390174" | "390175" | "390176" | "390177" | "390178" | "390179" 
            | "390180" | "390181" | "390182" | "390183" | "390184" | "390185" | "390186" | "390187" 
            | "390188" | "390189" | "390190" | "390191" | "390192" | "390193" | "390194" | "390195" 
            | "390196" | "390197" | "390198" | "390199" => crate::error::ErrorKind::NotNullViolation,
            
            // Syntax errors
            "1003" => crate::error::ErrorKind::Other,
            
            // Constraint violations
            "100072" => crate::error::ErrorKind::UniqueViolation,
            "100071" => crate::error::ErrorKind::NotNullViolation,
            "100070" => crate::error::ErrorKind::ForeignKeyViolation,
            "100069" => crate::error::ErrorKind::CheckViolation,
            
            // Connection errors
            "250001" | "250002" | "250003" | "250004" | "250005" => crate::error::ErrorKind::Other,
            
            _ => crate::error::ErrorKind::Other,
        }
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