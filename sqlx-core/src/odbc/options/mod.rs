use crate::connection::{ConnectOptions, LogSettings};
use crate::error::Error;
use futures_core::future::BoxFuture;
use log::LevelFilter;
use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;
use std::time::Duration;

use crate::odbc::OdbcConnection;

#[derive(Clone)]
pub struct OdbcConnectOptions {
    pub(crate) conn_str: String,
    pub(crate) log_settings: LogSettings,
}

impl OdbcConnectOptions {
    pub fn connection_string(&self) -> &str {
        &self.conn_str
    }
}

impl Debug for OdbcConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OdbcConnectOptions")
            .field("conn_str", &"<redacted>")
            .finish()
    }
}

impl FromStr for OdbcConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Use full string as ODBC connection string or DSN
        Ok(Self {
            conn_str: s.to_owned(),
            log_settings: LogSettings::default(),
        })
    }
}

impl ConnectOptions for OdbcConnectOptions {
    type Connection = OdbcConnection;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(OdbcConnection::establish(self))
    }

    fn log_statements(&mut self, level: LevelFilter) -> &mut Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(&mut self, level: LevelFilter, duration: Duration) -> &mut Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}
