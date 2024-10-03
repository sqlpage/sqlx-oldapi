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

        log::debug!(
            "Sending T-SQL PRELOGIN with encryption: {:?}",
            options.encrypt
        );

        stream
            .write_packet_and_flush(
                PacketType::PreLogin,
                PreLogin {
                    version: Version::default(),
                    encryption: options.encrypt,
                    instance: options.instance.clone(),

                    ..Default::default()
                },
            )
            .await?;

        let (_, packet) = stream.recv_packet().await?;
        let prelogin_response = PreLogin::decode(packet)?;

        if matches!(
            prelogin_response.encryption,
            Encrypt::Required | Encrypt::On
        ) {
            stream.setup_encryption().await?;
        } else if options.encrypt == Encrypt::Required {
            return Err(Error::Tls(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "TLS encryption required but not supported by server",
            ))));
        }

        // LOGIN7 defines the authentication rules for use between client and server

        stream
            .write_packet_and_flush(
                PacketType::Tds7Login,
                Login7 {
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
                },
            )
            .await?;

        loop {
            // NOTE: we should receive an [Error] message if something goes wrong, otherwise,
            //       all messages are mostly informational (ENVCHANGE, INFO, LOGINACK)

            match stream.recv_message().await? {
                Message::LoginAck(_) => {
                    // indicates that the login was successful
                    // no action is needed, we are just going to keep waiting till we hit <Done>
                }

                Message::Done(_) => {
                    break;
                }

                _ => {}
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
