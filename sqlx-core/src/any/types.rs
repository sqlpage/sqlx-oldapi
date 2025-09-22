//! Conversions between Rust and standard **SQL** types.
//!
//! # Types
//!
//! | Rust type                             | SQL type(s)                                          |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BOOLEAN                                              |
//! | `i16`                                 | SMALLINT                                             |
//! | `i32`                                 | INT                                                  |
//! | `i64`                                 | BIGINT                                               |
//! | `f32`                                 | FLOAT                                                |
//! | `f64`                                 | DOUBLE                                               |
//! | `&str`, [`String`]                    | VARCHAR, CHAR, TEXT                                  |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from SQL.
//!

// Type

impl_any_type!(bool);

impl_any_type!(i8);
impl_any_type!(i16);
impl_any_type!(i32);
impl_any_type!(i64);

impl_any_type!(f32);
impl_any_type!(f64);

impl_any_type!(str);
impl_any_type!(String);

impl_any_type!(u16);
impl_any_type!(u32);
impl_any_type!(u64);

// Encode

impl_any_encode!(bool);

impl_any_encode!(i8);
impl_any_encode!(i16);
impl_any_encode!(i32);
impl_any_encode!(i64);

impl_any_encode!(f32);
impl_any_encode!(f64);

impl_any_encode!(&'q str);
impl_any_encode!(String);

impl_any_encode!(u16);
impl_any_encode!(u32);
impl_any_encode!(u64);

// Decode

impl_any_decode!(bool);

impl_any_decode!(i8);
impl_any_decode!(i16);
impl_any_decode!(i32);
impl_any_decode!(i64);

impl_any_decode!(f32);
impl_any_decode!(f64);

impl_any_decode!(&'r str);
impl_any_decode!(String);

impl_any_decode!(u16);
impl_any_decode!(u32);
impl_any_decode!(u64);

// Conversions for Blob SQL types
// Type
impl_any_type!([u8]);
impl_any_type!(Vec<u8>);

// Encode
impl_any_encode!(&'q [u8]);
impl_any_encode!(Vec<u8>);

// Decode
impl_any_decode!(&'r [u8]);
impl_any_decode!(Vec<u8>);

// Conversions for Time SQL types
// Type
#[cfg(feature = "chrono")]
impl_any_type!(chrono::NaiveDate);
#[cfg(feature = "chrono")]
impl_any_type!(chrono::NaiveTime);
#[cfg(feature = "chrono")]
impl_any_type!(chrono::NaiveDateTime);
#[cfg(feature = "chrono")]
impl_any_type!(chrono::DateTime<chrono::offset::Utc>);
#[cfg(feature = "chrono")]
impl_any_type!(chrono::DateTime<chrono::offset::FixedOffset>);
#[cfg(feature = "chrono")]
impl_any_type!(chrono::DateTime<chrono::offset::Local>);

// Encode
#[cfg(feature = "chrono")]
impl_any_encode!(chrono::NaiveDate);
#[cfg(feature = "chrono")]
impl_any_encode!(chrono::NaiveTime);
#[cfg(feature = "chrono")]
impl_any_encode!(chrono::NaiveDateTime);
#[cfg(feature = "chrono")]
impl_any_encode!(chrono::DateTime<chrono::offset::Utc>);
#[cfg(feature = "chrono")]
impl_any_encode!(chrono::DateTime<chrono::offset::FixedOffset>);
#[cfg(feature = "chrono")]
impl_any_encode!(chrono::DateTime<chrono::offset::Local>);

// Decode
#[cfg(feature = "chrono")]
impl_any_decode!(chrono::NaiveDate);
#[cfg(feature = "chrono")]
impl_any_decode!(chrono::NaiveTime);
#[cfg(feature = "chrono")]
impl_any_decode!(chrono::NaiveDateTime);
#[cfg(feature = "chrono")]
impl_any_decode!(chrono::DateTime<chrono::offset::Utc>);
#[cfg(feature = "chrono")]
impl_any_decode!(chrono::DateTime<chrono::offset::FixedOffset>);
#[cfg(feature = "chrono")]
impl_any_decode!(chrono::DateTime<chrono::offset::Local>);

#[cfg(feature = "json")]
mod json_types {
    use serde_json::Value;
    impl_any_type!(Value);
    impl_any_encode!(Value);
    impl_any_decode!(Value);
}

#[cfg(feature = "bigdecimal")]
mod bigdecimal_types {
    use bigdecimal::BigDecimal;
    impl_any_type!(BigDecimal);
    impl_any_encode!(BigDecimal);
    impl_any_decode!(BigDecimal);
}

#[cfg(feature = "decimal")]
mod decimal_types {
    use rust_decimal::Decimal;
    impl_any_type!(Decimal);
    impl_any_encode!(Decimal);
    impl_any_decode!(Decimal);
}

#[cfg(feature = "uuid")]
mod uuid_types {
    use uuid::Uuid;
    impl_any_type!(Uuid);
    impl_any_encode!(Uuid);
    impl_any_decode!(Uuid);
}
