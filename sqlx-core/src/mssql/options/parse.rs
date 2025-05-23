use crate::error::Error;
use crate::mssql::protocol::pre_login::Encrypt;
use crate::mssql::MssqlConnectOptions;
use percent_encoding::percent_decode_str;
use std::str::FromStr;
use url::Url;

impl FromStr for MssqlConnectOptions {
    type Err = Error;

    /// Parse a connection string into a set of connection options.
    ///
    /// The connection string should be a valid URL with the following format:
    /// ```text
    /// mssql://[username[:password]@]host[:port][/database][?param1=value1&param2=value2...]
    /// ```
    ///
    /// Components:
    /// - `username`: The username for SQL Server authentication.
    /// - `password`: The password for SQL Server authentication.
    /// - `host`: The hostname or IP address of the SQL Server.
    /// - `port`: The port number (default is 1433).
    /// - `database`: The name of the database to connect to.
    ///
    /// Supported query parameters:
    /// - `instance`: SQL Server named instance.
    /// - `encrypt`: Controls connection encryption:
    ///   - `strict`: Requires encryption and validates the server certificate.
    ///   - `mandatory` or `true` or `yes`: Requires encryption but doesn't validate the server certificate.
    ///   - `optional` or `false` or `no`: Uses encryption if available, falls back to unencrypted.
    ///   - `not_supported`: No encryption.
    /// - `sslrootcert` or `ssl-root-cert` or `ssl-ca`: Path to the root certificate for validating the server's SSL certificate.
    /// - `trust_server_certificate`: When true, skips validation of the server's SSL certificate. Use with caution as it makes the connection vulnerable to man-in-the-middle attacks.
    /// - `hostname_in_certificate`: The hostname expected in the server's SSL certificate. Use this when the server's hostname doesn't match the certificate.
    /// - `packet_size`: Size of TDS packets in bytes. Larger sizes can improve performance but consume more memory on the server
    /// - `client_program_version`: Version number of the client program, sent to the server for logging purposes.
    /// - `client_pid`: Process ID of the client, sent to the server for logging purposes.
    /// - `hostname`: Name of the client machine, sent to the server for logging purposes.
    /// - `app_name`: Name of the client application, sent to the server for logging purposes.
    /// - `server_name`: Name of the server to connect to. Useful when connecting through a proxy or load balancer.
    /// - `client_interface_name`: Name of the client interface, sent to the server for logging purposes.
    /// - `language`: Sets the language for server messages. Affects date formats and system messages.
    ///
    /// Example:
    /// ```text
    /// mssql://user:pass@localhost:1433/mydb?encrypt=strict&app_name=MyApp&packet_size=4096
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: Url = s.parse().map_err(Error::config)?;
        let mut options = Self::new();

        if let Some(host) = url.host_str() {
            options = options.host(host);
        }

        if let Some(port) = url.port() {
            options = options.port(port);
        }

        let username = url.username();
        if !username.is_empty() {
            options = options.username(
                &*percent_decode_str(username)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        if let Some(password) = url.password() {
            options = options.password(
                &*percent_decode_str(password)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(path);
        }

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "instance" => {
                    options = options.instance(&*value);
                }
                "encrypt" => {
                    match value.to_lowercase().as_str() {
                        "strict" => options = options.encrypt(Encrypt::Required),
                        "mandatory" | "true" | "yes" => options = options.encrypt(Encrypt::On),
                        "optional" | "false" | "no" => options = options.encrypt(Encrypt::Off),
                        "not_supported" => options = options.encrypt(Encrypt::NotSupported),
                        _ => return Err(Error::config(MssqlInvalidOption(format!(
                            "encrypt={} is not a valid value for encrypt. Valid values are: strict, mandatory, optional, true, false, yes, no",
                            value
                        )))),
                    }
                }
                "sslrootcert" | "ssl-root-cert" | "ssl-ca" => {
                    options = options.ssl_root_cert(&*value);
                }
                "trust_server_certificate" => {
                    let trust = value.parse::<bool>().map_err(Error::config)?;
                    options = options.trust_server_certificate(trust);
                }
                "hostname_in_certificate" => {
                    options = options.hostname_in_certificate(&*value);
                }
                "packet_size" => {
                    let size = value.parse().map_err(Error::config)?;
                    options = options.requested_packet_size(size).map_err(|_| {
                        Error::config(MssqlInvalidOption(format!("packet_size={}", size)))
                    })?;
                }
                "client_program_version" => {
                    options = options.client_program_version(value.parse().map_err(Error::config)?)
                }
                "client_pid" => options = options.client_pid(value.parse().map_err(Error::config)?),
                "hostname" => options = options.hostname(&*value),
                "app_name" => options = options.app_name(&*value),
                "server_name" => options = options.server_name(&*value),
                "client_interface_name" => options = options.client_interface_name(&*value),
                "language" => options = options.language(&*value),
                _ => {
                    return Err(Error::config(MssqlInvalidOption(key.into())));
                }
            }
        }
        Ok(options)
    }
}

#[derive(Debug)]
struct MssqlInvalidOption(String);

impl std::fmt::Display for MssqlInvalidOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` is not a valid mssql connection option", self.0)
    }
}

impl std::error::Error for MssqlInvalidOption {}

#[test]
fn it_parses_username_with_at_sign_correctly() {
    let url = "mysql://user@hostname:password@hostname:5432/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!("user@hostname", &opts.username);
}

#[test]
fn it_parses_password_with_non_ascii_chars_correctly() {
    let url = "mysql://username:p@ssw0rd@hostname:5432/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!(Some("p@ssw0rd".into()), opts.password);
}
