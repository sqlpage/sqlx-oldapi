//! Conversions between Rust and **Snowflake** types.
//!
//! # Types
//!
//! | Rust type                             | Snowflake type(s)                                        |
//! |---------------------------------------|----------------------------------------------------------|
//! | `bool`                                | BOOLEAN                                                  |
//! | `i8`                                  | TINYINT                                                  |
//! | `i16`                                 | SMALLINT                                                 |
//! | `i32`                                 | INT, INTEGER                                             |
//! | `i64`                                 | BIGINT                                                   |
//! | `f32`                                 | FLOAT, FLOAT4, REAL                                     |
//! | `f64`                                 | DOUBLE, DOUBLE PRECISION, FLOAT8                        |
//! | `&str`, [`String`]                    | VARCHAR, CHAR, CHARACTER, STRING, TEXT                  |
//! | `&[u8]`, `Vec<u8>`                    | BINARY, VARBINARY                                       |

mod bool;
mod bytes;
mod float;
mod int;
mod str;
mod uint;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "uuid")]
mod uuid;

#[cfg(feature = "bigdecimal")]
mod bigdecimal;

#[cfg(feature = "decimal")]
mod decimal;

// Optional type support modules - only include if features are enabled
// TODO: Implement these when the corresponding features are needed
