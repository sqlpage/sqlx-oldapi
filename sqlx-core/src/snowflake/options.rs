use crate::connection::ConnectOptions;
use crate::error::Error;
use std::borrow::Cow;
use std::str::FromStr;
use url::Url;

/// Options for configuring a Snowflake connection.
#[derive(Debug, Clone)]
pub struct SnowflakeConnectOptions {
    pub(crate) account: String,
    pub(crate) warehouse: Option<String>,
    pub(crate) database: Option<String>,
    pub(crate) schema: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) username: String,
    pub(crate) password: Option<String>,
    pub(crate) private_key_path: Option<String>,
    pub(crate) private_key_data: Option<String>,
    pub(crate) passphrase: Option<String>,
    pub(crate) timeout: Option<std::time::Duration>,
}

/// SSL mode for Snowflake connections.
/// 
/// Snowflake always uses SSL, so this is mainly for future extensibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnowflakeSslMode {
    /// Always use SSL (default and only supported mode for Snowflake)
    Require,
}

impl Default for SnowflakeSslMode {
    fn default() -> Self {
        SnowflakeSslMode::Require
    }
}

impl SnowflakeConnectOptions {
    pub fn new() -> Self {
        Self {
            account: String::new(),
            warehouse: None,
            database: None,
            schema: None,
            role: None,
            username: String::new(),
            password: None,
            private_key_path: None,
            private_key_data: None,
            passphrase: None,
            timeout: None,
        }
    }

    pub fn account(mut self, account: impl Into<String>) -> Self {
        self.account = account.into();
        self
    }

    pub fn warehouse(mut self, warehouse: impl Into<String>) -> Self {
        self.warehouse = Some(warehouse.into());
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn private_key_path(mut self, path: impl Into<String>) -> Self {
        self.private_key_path = Some(path.into());
        self
    }

    pub fn private_key_data(mut self, data: impl Into<String>) -> Self {
        self.private_key_data = Some(data.into());
        self
    }

    pub fn passphrase(mut self, passphrase: impl Into<String>) -> Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    // Getter methods for testing
    pub fn get_account(&self) -> &str {
        &self.account
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_database(&self) -> Option<&str> {
        self.database.as_deref()
    }

    pub fn get_warehouse(&self) -> Option<&str> {
        self.warehouse.as_deref()
    }

    pub fn get_schema(&self) -> Option<&str> {
        self.schema.as_deref()
    }

    pub fn from_url(url: &Url) -> Result<Self, Error> {
        let mut options = SnowflakeConnectOptions::new();

        // Extract account from host (format: account.snowflakecomputing.com)
        if let Some(host) = url.host_str() {
            if let Some(account) = host.split('.').next() {
                options = options.account(account);
            }
        }

        // Extract username from URL
        if !url.username().is_empty() {
            options = options.username(url.username());
        }

        // Extract password from URL
        if let Some(password) = url.password() {
            options = options.password(password);
        }

        // Extract database from path
        let path = url.path();
        if !path.is_empty() && path != "/" {
            let database = path.trim_start_matches('/');
            if !database.is_empty() {
                options = options.database(database);
            }
        }

        // Extract query parameters
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "warehouse" => options = options.warehouse(value.as_ref()),
                "database" | "db" => options = options.database(value.as_ref()),
                "schema" => options = options.schema(value.as_ref()),
                "role" => options = options.role(value.as_ref()),
                "private_key_path" => options = options.private_key_path(value.as_ref()),
                "private_key_data" => options = options.private_key_data(value.as_ref()),
                "passphrase" => options = options.passphrase(value.as_ref()),
                _ => {}
            }
        }

        Ok(options)
    }
}

impl Default for SnowflakeConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for SnowflakeConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s).map_err(|e| Error::Configuration(e.into()))?;
        Self::from_url(&url)
    }
}

impl ConnectOptions for SnowflakeConnectOptions {
    type Connection = crate::snowflake::SnowflakeConnection;

    fn connect(&self) -> futures_core::future::BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async move {
            crate::snowflake::SnowflakeConnection::establish(self).await
        })
    }

    fn log_statements(&mut self, _level: log::LevelFilter) -> &mut Self {
        // TODO: implement statement logging
        self
    }

    fn log_slow_statements(
        &mut self,
        _level: log::LevelFilter,
        _duration: std::time::Duration,
    ) -> &mut Self {
        // TODO: implement slow statement logging
        self
    }
}