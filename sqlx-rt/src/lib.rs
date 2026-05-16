//! Core runtime support for SQLx. **Semver-exempt**, not for general use.

#[cfg(not(any(feature = "native-tls", feature = "rustls",)))]
compile_error!("one of the features ['native-tls', 'rustls'] must be enabled");

#[cfg(any(all(feature = "_tls-native-tls", feature = "_tls-rustls"),))]
compile_error!("only one of ['native-tls', 'rustls'] can be enabled");

#[cfg(any(feature = "native-tls", feature = "rustls"))]
mod rt_tokio;

#[cfg(feature = "_tls-native-tls")]
pub use native_tls;

//
// Tokio
//

#[cfg(any(feature = "native-tls", feature = "rustls"))]
pub use rt_tokio::*;
