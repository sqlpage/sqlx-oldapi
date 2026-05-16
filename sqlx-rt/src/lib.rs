//! Core runtime support for SQLx. **Semver-exempt**, not for general use.

#[cfg(not(any(
    feature = "runtime-actix-native-tls",
    feature = "runtime-tokio-native-tls",
    feature = "runtime-actix-rustls",
    feature = "runtime-tokio-rustls",
)))]
compile_error!(
    "one of the features ['runtime-actix-native-tls', 'runtime-tokio-native-tls', \
     'runtime-actix-rustls', 'runtime-tokio-rustls'] must be enabled"
);

#[cfg(any(
    all(feature = "_rt-actix", feature = "_rt-tokio"),
    all(feature = "_tls-native-tls", feature = "_tls-rustls"),
))]
compile_error!(
    "only one of ['runtime-actix-native-tls', 'runtime-tokio-native-tls', \
     'runtime-actix-rustls', 'runtime-tokio-rustls'] can be enabled"
);

#[cfg(any(feature = "_rt-tokio", feature = "_rt-actix"))]
mod rt_tokio;

#[cfg(feature = "_tls-native-tls")]
pub use native_tls;

//
// Actix *OR* Tokio
//

#[cfg(any(feature = "_rt-tokio", feature = "_rt-actix"))]
pub use rt_tokio::*;
