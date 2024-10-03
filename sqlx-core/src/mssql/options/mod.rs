use std::path::Path;

use super::protocol::pre_login::Encrypt;
use crate::{connection::LogSettings, net::CertificateInput};

mod connect;
mod parse;

/// Options and flags which can be used to configure a Microsoft SQL Server connection.
///
/// Connection strings should be in the form:
/// ```text
/// mssql://[username[:password]@]host/database[?instance=instance_name&packet_size=packet_size&client_program_version=client_program_version&client_pid=client_pid&hostname=hostname&app_name=app_name&server_name=server_name&client_interface_name=client_interface_name&language=language]
/// ```
#[derive(Debug, Clone)]
pub struct MssqlConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) database: String,
    pub(crate) password: Option<String>,
    pub(crate) instance: Option<String>,
    pub(crate) log_settings: LogSettings,
    pub(crate) client_program_version: u32,
    pub(crate) client_pid: u32,
    pub(crate) hostname: String,
    pub(crate) app_name: String,
    pub(crate) server_name: String,
    pub(crate) client_interface_name: String,
    pub(crate) language: String,
    /// Size in bytes of TDS packets to exchange with the server
    pub(crate) requested_packet_size: u32,
    pub(crate) encrypt: Encrypt,
    pub(crate) trust_server_certificate: bool,
    pub(crate) hostname_in_certificate: Option<String>,
    pub(crate) ssl_root_cert: Option<CertificateInput>,
}

impl Default for MssqlConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl MssqlConnectOptions {
    pub fn new() -> Self {
        Self {
            port: 1433,
            host: String::from("localhost"),
            database: String::from("master"),
            username: String::from("sa"),
            password: None,
            instance: None,
            log_settings: Default::default(),
            requested_packet_size: 4096,
            client_program_version: 0,
            client_pid: 0,
            hostname: String::new(),
            app_name: String::new(),
            server_name: String::new(),
            client_interface_name: String::new(),
            language: String::new(),
            encrypt: Encrypt::On,
            trust_server_certificate: true,
            hostname_in_certificate: None,
            ssl_root_cert: None,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_owned();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_owned();
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    pub fn database(mut self, database: &str) -> Self {
        self.database = database.to_owned();
        self
    }

    pub fn instance(mut self, instance: &str) -> Self {
        self.instance = Some(instance.to_owned());
        self
    }

    pub fn client_program_version(mut self, client_program_version: u32) -> Self {
        self.client_program_version = client_program_version;
        self
    }

    pub fn client_pid(mut self, client_pid: u32) -> Self {
        self.client_pid = client_pid;
        self
    }

    pub fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = hostname.to_owned();
        self
    }

    pub fn app_name(mut self, app_name: &str) -> Self {
        self.app_name = app_name.to_owned();
        self
    }

    pub fn server_name(mut self, server_name: &str) -> Self {
        self.server_name = server_name.to_owned();
        self
    }

    pub fn client_interface_name(mut self, client_interface_name: &str) -> Self {
        self.client_interface_name = client_interface_name.to_owned();
        self
    }

    pub fn language(mut self, language: &str) -> Self {
        self.language = language.to_owned();
        self
    }

    /// Size in bytes of TDS packets to exchange with the server.
    /// Returns an error if the size is smaller than 512 bytes
    pub fn requested_packet_size(mut self, size: u32) -> Result<Self, Self> {
        if size < 512 {
            Err(self)
        } else {
            self.requested_packet_size = size;
            Ok(self)
        }
    }

    pub fn encrypt(mut self, encrypt: Encrypt) -> Self {
        self.encrypt = encrypt;
        self
    }

    pub fn trust_server_certificate(mut self, trust: bool) -> Self {
        self.trust_server_certificate = trust;
        self
    }

    pub fn hostname_in_certificate(mut self, hostname: &str) -> Self {
        self.hostname_in_certificate = Some(hostname.to_owned());
        self
    }

    /// Sets the name of a file containing SSL certificate authority (CA) certificate(s).
    /// If the file exists, the server's certificate will be verified to be signed by
    /// one of these authorities.
    pub fn ssl_root_cert(mut self, cert: impl AsRef<Path>) -> Self {
        self.ssl_root_cert = Some(CertificateInput::File(cert.as_ref().to_path_buf()));
        self
    }
}
