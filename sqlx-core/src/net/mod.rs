mod socket;
mod tls;

#[allow(unused_imports)]
pub use socket::Socket;
#[allow(unused_imports)]
pub use tls::CertificateInput;
#[allow(unused_imports)]
pub use tls::MaybeTlsStream;
#[allow(unused_imports)]
pub use tls::TlsConfig;

type PollReadBuf<'a> = sqlx_rt::ReadBuf<'a>;

type PollReadOut = ();
