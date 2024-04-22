mod socket;
mod tls;

#[allow(unused_imports)] pub use socket::Socket;
pub use tls::CertificateInput;
#[allow(unused_imports)] pub use tls::MaybeTlsStream;

#[cfg(feature = "_rt-async-std")]
type PollReadBuf<'a> = [u8];

#[cfg(feature = "_rt-tokio")]
type PollReadBuf<'a> = sqlx_rt::ReadBuf<'a>;

#[cfg(feature = "_rt-async-std")]
type PollReadOut = usize;

#[cfg(feature = "_rt-tokio")]
type PollReadOut = ();
