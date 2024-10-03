use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::WebPkiServerVerifier;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{
    CertificateError, ClientConfig, DigitallySignedStruct, Error as TlsError, KeyLogFile,
    RootCertStore, SignatureScheme,
};
use std::io::{BufReader, Cursor};
use std::sync::Arc;

use crate::error::Error;

use super::TlsConfig;

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
            let data = ca.data().await?;
            let mut cursor = Cursor::new(data);

            for cert in rustls_pemfile::certs(&mut cursor) {
                cert_store
                    .add(cert.map_err(|err| Error::Tls(err.into()))?)
                    .map_err(|err| Error::Tls(err.into()))?;
            }
        }

        if tls_config.accept_invalid_hostnames {
            let verifier = WebPkiServerVerifier::builder(Arc::new(cert_store))
                .build()
                .map_err(|err| Error::Tls(err.into()))?;

            config
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoHostnameTlsVerifier { verifier }))
        } else {
            config.with_root_certificates(cert_store)
        }
    };

    // authentication using user's key and its associated certificate
    let mut config = match (tls_config.client_cert_path, tls_config.client_key_path) {
        (Some(cert_path), Some(key_path)) => {
            let cert_chain = certs_from_pem(cert_path.data().await?)?;
            let key_der = private_key_from_pem(key_path.data().await?)?;
            config
                .with_client_auth_cert(cert_chain, key_der)
                .map_err(Error::tls)?
        }
        (None, None) => config.with_no_client_auth(),
        (_, _) => {
            return Err(Error::Configuration(
                "user auth key and certs must be given together".into(),
            ))
        }
    };

    // When SSLKEYLOGFILE is set, write the TLS keys to a file for use with Wireshark
    config.key_log = Arc::new(KeyLogFile::new());

    Ok(Arc::new(config).into())
}

fn certs_from_pem(pem: Vec<u8>) -> std::io::Result<Vec<CertificateDer<'static>>> {
    let cur = Cursor::new(pem);
    let mut reader = BufReader::new(cur);
    rustls_pemfile::certs(&mut reader).collect()
}

fn private_key_from_pem(pem: Vec<u8>) -> Result<PrivateKeyDer<'static>, Error> {
    let cur = Cursor::new(pem);
    let mut reader = BufReader::new(cur);

    loop {
        match rustls_pemfile::read_one(&mut reader)? {
            Some(rustls_pemfile::Item::Sec1Key(key)) => return Ok(PrivateKeyDer::Sec1(key)),
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => return Ok(PrivateKeyDer::Pkcs8(key)),
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => return Ok(PrivateKeyDer::Pkcs1(key)),
            None => break,
            _ => {}
        }
    }

    Err(Error::Configuration("no keys found pem file".into()))
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
