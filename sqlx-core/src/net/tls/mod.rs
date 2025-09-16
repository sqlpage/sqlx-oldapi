#![allow(dead_code)]

use std::io;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use sqlx_rt::{AsyncRead, AsyncWrite, TlsStream};

use crate::error::Error;
use std::mem::replace;

/// X.509 Certificate input, either a file path or a PEM encoded inline certificate(s).
#[derive(Clone, Debug)]
pub enum CertificateInput {
    /// PEM encoded certificate(s)
    Inline(Vec<u8>),
    /// Path to a file containing PEM encoded certificate(s)
    File(PathBuf),
}

impl From<String> for CertificateInput {
    fn from(value: String) -> Self {
        let trimmed = value.trim();
        // Some heuristics according to https://tools.ietf.org/html/rfc7468
        if trimmed.starts_with("-----BEGIN CERTIFICATE-----")
            && trimmed.contains("-----END CERTIFICATE-----")
        {
            CertificateInput::Inline(value.as_bytes().to_vec())
        } else {
            CertificateInput::File(PathBuf::from(value))
        }
    }
}

impl CertificateInput {
    async fn data(&self) -> Result<Vec<u8>, Error> {
        use sqlx_rt::fs;
        match self {
            CertificateInput::Inline(v) => Ok(v.clone()),
            CertificateInput::File(path) => fs::read(path).await.map_err(|e| {
                Error::tls(CertificateReadError {
                    path: path.clone(),
                    source: e,
                })
            }),
        }
    }
}

impl std::fmt::Display for CertificateInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CertificateInput::Inline(v) => write!(f, "{}", String::from_utf8_lossy(v.as_slice())),
            CertificateInput::File(path) => write!(f, "file: {}", path.display()),
        }
    }
}

#[derive(Debug)]
struct CertificateReadError {
    path: PathBuf,
    source: io::Error,
}

impl std::fmt::Display for CertificateReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to read certificate file '{}': {}",
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for CertificateReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub struct TlsConfig<'a> {
    pub accept_invalid_certs: bool,
    pub accept_invalid_hostnames: bool,
    pub hostname: &'a str,
    pub root_cert_path: Option<&'a CertificateInput>,
    pub client_cert_path: Option<&'a CertificateInput>,
    pub client_key_path: Option<&'a CertificateInput>,
}

#[cfg(feature = "_tls-rustls")]
mod rustls;

pub enum MaybeTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    Raw(S),
    Tls(TlsStream<S>),
    Upgrading,
}

impl<S> MaybeTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    #[inline]
    pub fn is_tls(&self) -> bool {
        matches!(self, Self::Tls(_))
    }

    pub async fn upgrade(&mut self, config: TlsConfig<'_>) -> Result<(), Error> {
        let host = config.hostname;
        let connector = configure_tls_connector(config).await?;

        let stream = match replace(self, MaybeTlsStream::Upgrading) {
            MaybeTlsStream::Raw(stream) => stream,

            MaybeTlsStream::Tls(_) => {
                // ignore upgrade, we are already a TLS connection
                return Ok(());
            }

            MaybeTlsStream::Upgrading => {
                // we previously failed to upgrade and now hold no connection
                // this should only happen from an internal misuse of this method
                return Err(Error::Io(io::ErrorKind::ConnectionAborted.into()));
            }
        };

        #[cfg(feature = "_tls-rustls")]
        let host = ::rustls::pki_types::ServerName::try_from(host)
            .map_err(|err| Error::Tls(err.into()))?
            .to_owned();

        *self = MaybeTlsStream::Tls(connector.connect(host, stream).await?);

        Ok(())
    }

    pub fn downgrade(&mut self) -> Result<(), Error> {
        match replace(self, MaybeTlsStream::Upgrading) {
            MaybeTlsStream::Tls(stream) => {
                #[cfg(feature = "_tls-rustls")]
                {
                    let raw = stream.into_inner().0;
                    *self = MaybeTlsStream::Raw(raw);
                    Ok(())
                }

                #[cfg(feature = "_tls-native-tls")]
                {
                    let _ = stream; // Use the variable to avoid warning
                    return Err(Error::tls("No way to downgrade a native-tls stream, use rustls instead, or never disable tls"));
                }
            }

            MaybeTlsStream::Raw(stream) => {
                *self = MaybeTlsStream::Raw(stream);
                Ok(())
            }

            MaybeTlsStream::Upgrading => Err(Error::Io(io::ErrorKind::ConnectionAborted.into())),
        }
    }
}

#[cfg(feature = "_tls-native-tls")]
async fn configure_tls_connector(config: TlsConfig<'_>) -> Result<sqlx_rt::TlsConnector, Error> {
    use sqlx_rt::native_tls::{Certificate, Identity, TlsConnector};

    let mut builder = TlsConnector::builder();
    builder
        .danger_accept_invalid_certs(config.accept_invalid_certs)
        .danger_accept_invalid_hostnames(config.accept_invalid_hostnames);

    if !config.accept_invalid_certs {
        if let Some(ca) = config.root_cert_path {
            let data = ca.data().await?;
            let cert = Certificate::from_pem(&data)?;

            builder.add_root_certificate(cert);
        }
    }

    // authentication using user's key-file and its associated certificate
    if let (Some(cert_path), Some(key_path)) = (config.client_cert_path, config.client_key_path) {
        let cert_path = cert_path.data().await?;
        let key_path = key_path.data().await?;
        let identity = Identity::from_pkcs8(&cert_path, &key_path).map_err(Error::tls)?;
        builder.identity(identity);
    }

    #[cfg(not(feature = "_rt-async-std"))]
    let connector = builder.build()?.into();

    #[cfg(feature = "_rt-async-std")]
    let connector = builder.into();

    Ok(connector)
}

#[cfg(feature = "_tls-rustls")]
use self::rustls::configure_tls_connector;

impl<S> AsyncRead for MaybeTlsStream<S>
where
    S: Unpin + AsyncWrite + AsyncRead,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut super::PollReadBuf<'_>,
    ) -> Poll<io::Result<super::PollReadOut>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_read(cx, buf),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_read(cx, buf),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }
}

impl<S> AsyncWrite for MaybeTlsStream<S>
where
    S: Unpin + AsyncWrite + AsyncRead,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_write(cx, buf),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_write(cx, buf),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_flush(cx),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_flush(cx),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }

    #[cfg(feature = "_rt-tokio")]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_shutdown(cx),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_shutdown(cx),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }

    #[cfg(feature = "_rt-async-std")]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_close(cx),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_close(cx),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }
}

impl<S> Deref for MaybeTlsStream<S>
where
    S: Unpin + AsyncWrite + AsyncRead,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeTlsStream::Raw(s) => s,

            #[cfg(feature = "_tls-rustls")]
            MaybeTlsStream::Tls(s) => s.get_ref().0,

            #[cfg(all(feature = "_rt-async-std", feature = "_tls-native-tls"))]
            MaybeTlsStream::Tls(s) => s.get_ref(),

            #[cfg(all(not(feature = "_rt-async-std"), feature = "_tls-native-tls"))]
            MaybeTlsStream::Tls(s) => s.get_ref().get_ref().get_ref(),

            MaybeTlsStream::Upgrading => {
                panic!("{}", io::Error::from(io::ErrorKind::ConnectionAborted))
            }
        }
    }
}

impl<S> DerefMut for MaybeTlsStream<S>
where
    S: Unpin + AsyncWrite + AsyncRead,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeTlsStream::Raw(s) => s,

            #[cfg(feature = "_tls-rustls")]
            MaybeTlsStream::Tls(s) => s.get_mut().0,

            #[cfg(all(feature = "_rt-async-std", feature = "_tls-native-tls"))]
            MaybeTlsStream::Tls(s) => s.get_mut(),

            #[cfg(all(not(feature = "_rt-async-std"), feature = "_tls-native-tls"))]
            MaybeTlsStream::Tls(s) => s.get_mut().get_mut().get_mut(),

            MaybeTlsStream::Upgrading => {
                panic!("{}", io::Error::from(io::ErrorKind::ConnectionAborted))
            }
        }
    }
}
