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

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "time")]
mod time;

#[cfg(feature = "uuid")]
mod uuid;

#[cfg(feature = "json")]
mod json;