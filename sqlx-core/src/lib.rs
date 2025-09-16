//! Core of SQLx, the rust SQL toolkit.
//! Not intended to be used directly.
#![recursion_limit = "512"]
#![warn(future_incompatible, rust_2018_idioms)]
#![allow(
    clippy::needless_doctest_main,
    clippy::type_complexity,
    dead_code,
    // Common style warnings that are numerous in the codebase
    clippy::explicit_auto_deref,
    clippy::needless_borrow,
    clippy::needless_return,
    clippy::redundant_closure,
    clippy::question_mark,
    clippy::unnecessary_cast,
    clippy::from_iter_instead_of_collect,
    clippy::doc_lazy_continuation,
    clippy::legacy_numeric_constants,
    clippy::should_implement_trait,
    clippy::derivable_impls,
    clippy::manual_map,
    clippy::nonminimal_bool,
    clippy::extra_unused_lifetimes,
    clippy::deref_by_slicing,
    clippy::let_underscore_future,
    clippy::needless_borrowed_reference,
    clippy::map_clone,
    clippy::large_enum_variant,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::multiple_bound_locations,
    clippy::unnecessary_map_or,
    clippy::extra_unused_type_parameters,
    clippy::iter_cloned_collect,
    clippy::io_other_error,
    clippy::needless_borrows_for_generic_args,
    clippy::is_digit_ascii_radix,
    clippy::len_zero,
    clippy::manual_is_multiple_of,
    clippy::while_let_on_iterator,
    clippy::wrong_self_convention,
    clippy::useless_conversion,
    clippy::ptr_arg,
    clippy::clone_on_copy,
    clippy::explicit_counter_loop,
    clippy::manual_inspect,
    clippy::len_without_is_empty,
    clippy::borrow_deref_ref,
    clippy::get_first,
    clippy::enum_variant_names,
    clippy::let_and_return,
    clippy::needless_option_as_deref,
    clippy::op_ref,
    clippy::drop_non_drop,
    clippy::bool_assert_comparison,
    clippy::empty_line_after_doc_comments,
    clippy::single_char_add_str,
    clippy::let_unit_value,
    clippy::unit_arg,
    clippy::result_large_err,
    clippy::needless_range_loop,
    clippy::manual_div_ceil,
    clippy::manual_range_patterns,
    clippy::never_loop,
    clippy::module_inception,
    clippy::unwrap_or_default,
    clippy::zero_prefixed_literal
)]
// Note: Cast warnings are allowed on a case-by-case basis with explicit #[allow(...)]
// This ensures we're aware of potential issues with numeric conversions
// See `clippy.toml` at the workspace root
#![deny(clippy::disallowed_methods)]
//
// Allows an API be documented as only available in some specific platforms.
// <https://doc.rust-lang.org/unstable-book/language-features/doc-cfg.html>
#![cfg_attr(docsrs, feature(doc_cfg))]
//
// When compiling with support for SQLite we must allow some unsafe code in order to
// interface with the inherently unsafe C module. This unsafe code is contained
// to the sqlite module.
#![cfg_attr(feature = "sqlite", deny(unsafe_code))]
#![cfg_attr(not(feature = "sqlite"), forbid(unsafe_code))]

#[cfg(feature = "bigdecimal")]
extern crate bigdecimal_ as bigdecimal;

#[macro_use]
mod ext;

#[macro_use]
pub mod error;

#[macro_use]
pub mod arguments;

#[macro_use]
pub mod pool;

pub mod connection;

#[macro_use]
pub mod transaction;

#[macro_use]
pub mod encode;

#[macro_use]
pub mod decode;

#[macro_use]
pub mod types;

#[macro_use]
pub mod query;

#[macro_use]
pub mod acquire;

#[macro_use]
pub mod column;

#[macro_use]
pub mod statement;

mod common;
pub use either::Either;
pub mod database;
pub mod describe;
pub mod executor;
pub mod from_row;
mod io;
mod logger;
mod net;
pub mod query_as;
pub mod query_builder;
pub mod query_scalar;
pub mod row;
pub mod type_info;
pub mod value;

#[cfg(feature = "migrate")]
pub mod migrate;

#[cfg(all(
    any(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite"
    ),
    feature = "any"
))]
pub mod any;

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub mod postgres;

#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub mod sqlite;

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub mod mysql;

#[cfg(feature = "mssql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mssql")))]
pub mod mssql;

// Implements test support with automatic DB management.
#[cfg(feature = "migrate")]
pub mod testing;

pub use sqlx_rt::test_block_on;

/// sqlx uses ahash for increased performance, at the cost of reduced DoS resistance.
use ahash::AHashMap as HashMap;
//type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
