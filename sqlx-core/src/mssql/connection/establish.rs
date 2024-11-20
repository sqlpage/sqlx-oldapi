use crate::common::StatementCache;
use crate::error::Error;
use crate::io::Decode;
use crate::mssql::connection::stream::MssqlStream;
use crate::mssql::protocol::login::Login7;
use crate::mssql::protocol::message::Message;
use crate::mssql::protocol::packet::PacketType;
use crate::mssql::protocol::pre_login::{Encrypt, PreLogin, Version};
use crate::mssql::{MssqlConnectOptions, MssqlConnection};

impl MssqlConnection {
    pub(crate) async fn establish(options: &MssqlConnectOptions) -> Result<Self, Error> {
        let mut stream: MssqlStream = MssqlStream::connect(options).await?;

        // Send PRELOGIN to set up the context for login. The server should immediately
        // respond with a PRELOGIN message of its own.

        // TODO: Encryption
        // TODO: Send the version of SQLx over

        let prelogin_packet = PreLogin {
            version: Version::default(),
            encryption: options.encrypt,
            instance: options.instance.clone(),
            ..Default::default()
        };

        log::debug!("Sending T-SQL PRELOGIN with encryption: {prelogin_packet:?}");
        stream
            .write_packet_and_flush(PacketType::PreLogin, prelogin_packet)
            .await?;

        let (_, packet) = stream.recv_packet().await?;

        let prelogin_response = PreLogin::decode(packet)?;
        log::debug!("Received PRELOGIN response: {:?}", prelogin_response);

        let mut disable_encryption_after_login = false;

        match (options.encrypt, prelogin_response.encryption) {
            (Encrypt::Required | Encrypt::On, Encrypt::Required | Encrypt::On) => {
                log::trace!("Mssql login phase and data packets encrypted");
                stream.setup_encryption().await?;
            }
            (Encrypt::Required, Encrypt::Off | Encrypt::NotSupported) => {
                return Err(Error::Tls(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "TLS encryption required but not supported by server",
                ))));
            }
            (Encrypt::Off, _) | (_, Encrypt::Off) => {
                log::info!("Mssql login phase encrypted, but data packets will be unencrypted");
                stream.setup_encryption().await?;
                disable_encryption_after_login = true;
            }
            (Encrypt::NotSupported, _) | (_, Encrypt::NotSupported) => {
                log::warn!("Mssql: fully unencrypted connection - will send plaintext password!");
            }
        }

        // LOGIN7 defines the authentication rules for use between client and server

        let login_packet = Login7 {
            // FIXME: use a version constant
            version: 0x74000004, // SQL Server 2012 - SQL Server 2019
            client_program_version: options.client_program_version,
            client_pid: options.client_pid,
            packet_size: options.requested_packet_size, // max allowed size of TDS packet
            hostname: &options.hostname,
            username: &options.username,
            password: options.password.as_deref().unwrap_or_default(),
            app_name: &options.app_name,
            server_name: &options.server_name,
            client_interface_name: &options.client_interface_name,
            language: &options.language,
            database: &*options.database,
            client_id: [0; 6],
        };

        log::debug!("Sending LOGIN7 packet: {login_packet:?}");
        stream
            .write_packet_and_flush(PacketType::Tds7Login, login_packet)
            .await?;

        log::debug!("Waiting for LOGINACK or DONE");

        if disable_encryption_after_login {
            log::debug!("Disabling encryption after login");
            stream.disable_encryption().await?;
        }

        loop {
            // NOTE: we should receive an [Error] message if something goes wrong, otherwise,
            //       all messages are mostly informational (ENVCHANGE, INFO, LOGINACK)

            match stream.recv_message().await? {
                Message::LoginAck(_) => {
                    // indicates that the login was successful
                    // no action is needed, we are just going to keep waiting till we hit <Done>
                    log::debug!("Received LoginAck");
                }

                Message::Done(_) => {
                    log::debug!("Pre-Login phase completed");
                    break;
                }

                other_msg => {
                    log::debug!("Ignoring unexpected pre-login message: {:?}", other_msg);
                }
            }
        }

        // FIXME: Do we need to expose the capacity count here? It's not tied to
        //        server-side resources but just .prepare() calls which return
        //        client-side data.

        Ok(Self {
            stream,
            cache_statement: StatementCache::new(1024),
            log_settings: options.log_settings.clone(),
        })
    }
}
