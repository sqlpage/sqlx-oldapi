//! Core runtime support for SQLx. **Semver-exempt**, not for general use.

#[cfg(not(any(feature = "native-tls", feature = "rustls")))]
compile_error!("one of the features ['native-tls', 'rustls'] must be enabled");

#[cfg(all(feature = "_tls-native-tls", feature = "_tls-rustls"))]
compile_error!("only one of ['native-tls', 'rustls'] can be enabled");

mod runtime;

#[cfg(feature = "_tls-native-tls")]
pub use native_tls;

pub use runtime::*;
