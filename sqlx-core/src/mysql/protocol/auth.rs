use std::str::FromStr;

use crate::error::Error;

#[derive(Debug, Copy, Clone)]
pub enum AuthPlugin {
    MySqlNative,
    CachingSha2,
    Sha256,
}

impl AuthPlugin {
    pub(crate) fn name(self) -> &'static str {
        match self {
            AuthPlugin::MySqlNative => "mysql_native_password",
            AuthPlugin::CachingSha2 => "caching_sha2_password",
            AuthPlugin::Sha256 => "sha256_password",
        }
    }
}

impl FromStr for AuthPlugin {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mysql_native_password" => Ok(AuthPlugin::MySqlNative),
            "caching_sha2_password" => Ok(AuthPlugin::CachingSha2),
            "sha256_password" => Ok(AuthPlugin::Sha256),

            _ => Err(err_protocol!("unknown authentication plugin: {}", s)),
        }
    }
}
