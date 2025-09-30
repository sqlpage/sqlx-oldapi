use crate::connection::{ConnectOptions, LogSettings};
use crate::error::Error;
use futures_core::future::BoxFuture;
use log::LevelFilter;
use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;
use std::time::Duration;

use crate::odbc::OdbcConnection;

/// Configuration for ODBC buffer settings that control memory usage and performance characteristics.
///
/// These settings affect how SQLx fetches and processes data from ODBC data sources. Careful tuning
/// of these parameters can significantly impact memory usage and query performance.
///
/// # Buffered vs Unbuffered Mode
///
/// The buffer settings control two different data fetching strategies based on `max_column_size`:
///
/// **Buffered Mode** (when `max_column_size` is `Some(value)`):
/// - Fetches rows in batches using columnar buffers for better performance with large result sets
/// - Pre-allocates memory for all columns and rows in each batch using `ColumnarAnyBuffer`
/// - WARNING: Long textual and binary field data will be truncated to `max_column_size`
/// - Better for high-throughput scenarios where memory usage is acceptable
/// - Uses `batch_size` to control how many rows are fetched at once
///
/// **Unbuffered Mode** (when `max_column_size` is `None`):
/// - Fetches rows in batches using the `next_row()` method for memory efficiency
/// - Processes rows in batches of `batch_size` but without pre-allocating columnar buffers
/// - No data truncation - handles variable-sized data dynamically
/// - More memory efficient than buffered mode but potentially slower
/// - Better for scenarios where data sizes vary significantly or memory is constrained
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OdbcBufferSettings {
    /// The number of rows to fetch in each batch during bulk operations.
    ///
    /// **Performance Impact:**
    /// - Higher values reduce the number of round-trips to the database but increase memory usage
    /// - Lower values reduce memory usage but may increase latency due to more frequent fetches
    /// - Typical range: 32-512 rows
    /// - Used in both buffered and unbuffered modes
    ///
    /// **Memory Impact:**
    /// - In buffered mode: Each batch allocates buffers for `batch_size * number_of_columns` cells
    /// - In unbuffered mode: Each batch processes `batch_size` rows without pre-allocation
    /// - For wide result sets, this can consume significant memory
    ///
    /// **Default:** 128 rows
    pub batch_size: usize,

    /// The maximum size (in characters) for text and binary columns when the database doesn't specify a length.
    ///
    /// **Performance Impact:**
    /// - When `Some(value)`: Enables buffered mode with batch fetching and pre-allocated buffers
    /// - When `None`: Enables unbuffered mode with batched row-by-row processing
    /// - Higher values ensure large text fields are fully captured but increase memory allocation
    /// - Lower values may truncate data but reduce memory pressure
    /// - Affects VARCHAR, NVARCHAR, TEXT, and BLOB column types
    ///
    /// **Memory Impact:**
    /// - When `Some(value)`: Directly controls buffer size for variable-length columns
    /// - When `None`: Minimal memory allocation per row, no truncation
    /// - Setting too high can waste memory; setting too low can cause data truncation
    /// - Consider your data characteristics when tuning this value
    ///
    /// **Default:** Some(4096)
    pub max_column_size: Option<usize>,
}

impl Default for OdbcBufferSettings {
    fn default() -> Self {
        Self {
            batch_size: 64,
            max_column_size: None,
        }
    }
}

#[derive(Clone)]
pub struct OdbcConnectOptions {
    pub(crate) conn_str: String,
    pub(crate) log_settings: LogSettings,
    pub(crate) buffer_settings: OdbcBufferSettings,
}

impl OdbcConnectOptions {
    pub fn connection_string(&self) -> &str {
        &self.conn_str
    }

    /// Sets the buffer configuration for this connection.
    ///
    /// The buffer settings control memory usage and performance characteristics
    /// when fetching data from ODBC data sources.
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::str::FromStr;
    /// use sqlx_core_oldapi::odbc::{OdbcConnectOptions, OdbcBufferSettings};
    ///
    /// let mut opts = OdbcConnectOptions::from_str("DSN=MyDataSource")?;
    ///
    /// // Configure for high-throughput buffered mode
    /// opts.buffer_settings(OdbcBufferSettings {
    ///     batch_size: 256,
    ///     max_column_size: Some(2048),
    /// });
    ///
    /// // Or configure for unbuffered mode
    /// opts.buffer_settings(OdbcBufferSettings {
    ///     batch_size: 128,  // batch_size is ignored in unbuffered mode
    ///     max_column_size: None,
    /// });
    /// # Ok::<(), sqlx_core_oldapi::error::Error>(())
    /// ```
    pub fn buffer_settings(&mut self, settings: OdbcBufferSettings) -> &mut Self {
        self.buffer_settings = settings;
        self
    }

    /// Sets the batch size for bulk fetch operations.
    ///
    /// This controls how many rows are fetched at once during query execution.
    /// Higher values can improve performance for large result sets but use more memory.
    /// Only used when `max_column_size` is `Some(value)` (buffered mode).
    ///
    /// # Panics
    /// Panics if `batch_size` is 0.
    pub fn batch_size(&mut self, batch_size: usize) -> &mut Self {
        assert!(batch_size > 0, "batch_size must be greater than 0");
        self.buffer_settings.batch_size = batch_size;
        self
    }

    /// Sets the maximum column size for text and binary data.
    ///
    /// This controls the buffer size allocated for columns when the database
    /// doesn't specify a maximum length. Larger values ensure complete data
    /// capture but increase memory usage.
    ///
    /// - When set to `Some(value)`: Enables buffered mode with batch fetching
    /// - When set to `None`: Enables unbuffered mode with row-by-row processing
    pub fn max_column_size(&mut self, max_column_size: Option<usize>) -> &mut Self {
        self.buffer_settings.max_column_size = max_column_size;
        self
    }

    /// Returns the current buffer settings for this connection.
    pub fn buffer_settings_ref(&self) -> &OdbcBufferSettings {
        &self.buffer_settings
    }
}

impl Debug for OdbcConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OdbcConnectOptions")
            .field("conn_str", &"<redacted>")
            .field("buffer_settings", &self.buffer_settings)
            .finish()
    }
}

impl FromStr for OdbcConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept forms:
        // - "odbc:DSN=Name;..." -> strip scheme
        // - "odbc:Name" -> interpret as DSN
        // - "DSN=Name;..." or full ODBC connection string
        let mut t = s.trim();
        if let Some(rest) = t.strip_prefix("odbc:") {
            t = rest;
        }
        let conn_str = if t.contains('=') {
            // Looks like an ODBC key=value connection string
            t.to_string()
        } else {
            // Bare DSN name
            format!("DSN={}", t)
        };

        Ok(Self {
            conn_str,
            log_settings: LogSettings::default(),
            buffer_settings: OdbcBufferSettings::default(),
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
