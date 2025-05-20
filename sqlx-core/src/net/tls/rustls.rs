use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::{VerifierBuilderError, WebPkiServerVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{
    CertificateError, ClientConfig, DigitallySignedStruct, Error as TlsError, KeyLogFile,
    RootCertStore, SignatureScheme,
};
use std::io::{BufReader, Cursor};
use std::sync::Arc;

use crate::error::Error;

use super::TlsConfig;

#[derive(Debug, thiserror::Error)]
pub enum RustlsError {
    #[error("failed to add root certificate from {path_desc} to trust store: {source}")]
    AddRootCert {
        path_desc: String,
        #[source]
        source: rustls::Error,
    },
    #[error("failed to parse PEM certificate from {file_description}: {source}")]
    ParsePemCert {
        file_description: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to build TLS verifier, ensure CA certificates are valid and correctly configured: {source}")]
    BuildVerifier {
        #[source]
        source: VerifierBuilderError,
    },
    #[error("failed to parse client certificate from {file_description}: {source}")]
    ParseClientCert {
        file_description: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse client private key from {file_description}: {source}")]
    ParseClientKey {
        file_description: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to set client authentication using certificate from {cert_path_desc} and private key from {key_path_desc}; ensure the certificate and key are valid: {source}")]
    ClientAuthCert {
        cert_path_desc: String,
        key_path_desc: String,
        #[source]
        source: TlsError,
    },
    #[error("no supported private keys (Sec1, Pkcs8, Pkcs1) found in the provided PEM data for the client private key")]
    NoKeysFound,
    #[error("TLS configuration error: {0}. Please check your TLS settings, including paths to certificates and keys, and ensure they are correctly specified.")]
    Configuration(String),
}

impl From<RustlsError> for Error {
    fn from(err: RustlsError) -> Self {
        Error::Tls(Box::new(err))
    }
}

pub async fn configure_tls_connector(
    tls_config: TlsConfig<'_>,
) -> Result<sqlx_rt::TlsConnector, Error> {
    let config = ClientConfig::builder();
    let config = if tls_config.accept_invalid_certs {
        config
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(DummyTlsVerifier))
    } else {
        let mut cert_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect(),
        };

        if let Some(ca) = tls_config.root_cert_path {
            let path_description = ca.to_string();
            let data = ca.data().await.map_err(|e| RustlsError::ParsePemCert {
                file_description: path_description.clone(),
                source: std::io::Error::new(std::io::ErrorKind::Other, e),
            })?;
            let mut cursor = Cursor::new(data);

            for cert_result in rustls_pemfile::certs(&mut cursor) {
                let cert = cert_result.map_err(|e| RustlsError::ParsePemCert {
                    file_description: path_description.clone(),
                    source: e,
                })?;
                cert_store.add(cert).map_err(|e| RustlsError::AddRootCert {
                    path_desc: path_description.clone(),
                    source: e,
                })?;
            }
        }

        if tls_config.accept_invalid_hostnames {
            let verifier = WebPkiServerVerifier::builder(Arc::new(cert_store))
                .build()
                .map_err(|e| RustlsError::BuildVerifier { source: e })?;

            config
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoHostnameTlsVerifier { verifier }))
        } else {
            config.with_root_certificates(cert_store)
        }
    };

    let mut config = match (tls_config.client_cert_path, tls_config.client_key_path) {
        (Some(cert_path), Some(key_path)) => {
            let cert_file_desc = cert_path.to_string();
            let key_file_desc = key_path.to_string();

            let cert_chain = certs_from_pem(cert_path.data().await.map_err(|e| {
                RustlsError::ParseClientCert {
                    file_description: cert_file_desc.clone(),
                    source: std::io::Error::new(std::io::ErrorKind::Other, e),
                }
            })?)?;
            let key_der = private_key_from_pem(key_path.data().await.map_err(|e| {
                RustlsError::ParseClientKey {
                    file_description: key_file_desc.clone(),
                    source: std::io::Error::new(std::io::ErrorKind::Other, e),
                }
            })?)?;
            config
                .with_client_auth_cert(cert_chain, key_der)
                .map_err(|e| RustlsError::ClientAuthCert {
                    cert_path_desc: cert_file_desc,
                    key_path_desc: key_file_desc,
                    source: e,
                })?
        }
        (None, None) => config.with_no_client_auth(),
        (_, _) => {
            return Err(RustlsError::Configuration(
                "user auth key and certs must be given together".into(),
            )
            .into())
        }
    };

    config.key_log = Arc::new(KeyLogFile::new());

    Ok(Arc::new(config).into())
}

fn certs_from_pem(pem: Vec<u8>) -> Result<Vec<CertificateDer<'static>>, RustlsError> {
    let cur = Cursor::new(pem);
    let mut reader = BufReader::new(cur);
    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| RustlsError::ParsePemCert {
            file_description: String::from("PEM data"),
            source: e,
        })
}

fn private_key_from_pem(pem: Vec<u8>) -> Result<PrivateKeyDer<'static>, RustlsError> {
    let cur = Cursor::new(pem);
    let mut reader = BufReader::new(cur);

    loop {
        match rustls_pemfile::read_one(&mut reader).map_err(|e| RustlsError::ParseClientKey {
            file_description: String::from("PEM data"),
            source: e,
        })? {
            Some(rustls_pemfile::Item::Sec1Key(key)) => return Ok(PrivateKeyDer::Sec1(key)),
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => return Ok(PrivateKeyDer::Pkcs8(key)),
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => return Ok(PrivateKeyDer::Pkcs1(key)),
            None => break,
            _ => {}
        }
    }

    Err(RustlsError::NoKeysFound)
}

#[derive(Debug)]
struct DummyTlsVerifier;

impl ServerCertVerifier for DummyTlsVerifier {
    // Required methods
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}

#[derive(Debug)]
pub struct NoHostnameTlsVerifier {
    verifier: Arc<WebPkiServerVerifier>,
}

impl ServerCertVerifier for NoHostnameTlsVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: rustls::pki_types::UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        remove_hostname_error(
            self.verifier.verify_server_cert(
                end_entity,
                intermediates,
                server_name,
                ocsp_response,
                now,
            ),
            ServerCertVerified::assertion(),
        )
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        remove_hostname_error(
            self.verifier.verify_tls12_signature(message, cert, dss),
            HandshakeSignatureValid::assertion(),
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        remove_hostname_error(
            self.verifier.verify_tls12_signature(message, cert, dss),
            HandshakeSignatureValid::assertion(),
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.verifier.supported_verify_schemes()
    }
}

fn remove_hostname_error<O>(r: Result<O, TlsError>, ok: O) -> Result<O, TlsError> {
    match r {
        Err(TlsError::InvalidCertificate(CertificateError::NotValidForName)) => Ok(ok),
        res => res,
    }
}
