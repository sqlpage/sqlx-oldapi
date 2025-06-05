#![allow(clippy::large_enum_variant)]
#![cfg_attr(
    not(any(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        feature = "offline"
    )),
    allow(dead_code, unused_macros, unused_imports)
)]
#![cfg_attr(
    any(sqlx_macros_unstable, procmacro2_semver_exempt),
    feature(track_path, proc_macro_tracked_env)
)]
extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, DeriveInput, ItemFn, LitStr, Meta, Token};

type Error = Box<dyn std::error::Error>;

type Result<T> = std::result::Result<T, Error>;

mod common;
mod database;
mod derives;
mod query;

// The compiler gives misleading help messages about `#[cfg(test)]` when this is just named `test`.
mod test_attr;

#[cfg(feature = "migrate")]
mod migrate;

struct ArgsParser(Punctuated<Meta, Token![,]>);

impl Parse for ArgsParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ArgsParser(Punctuated::<Meta, Token![,]>::parse_terminated(
            input,
        )?))
    }
}

#[proc_macro]
pub fn expand_query(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query::QueryMacroInput);

    match query::expand_input(input) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(::std::compile_error!(#msg)).into()
            }
        }
    }
}

#[proc_macro_derive(Encode, attributes(sqlx))]
pub fn derive_encode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_encode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Decode, attributes(sqlx))]
pub fn derive_decode(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Type, attributes(sqlx))]
pub fn derive_type(tokenstream: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokenstream as syn::DeriveInput);
    match derives::expand_derive_type_encode_decode(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(FromRow, attributes(sqlx))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match derives::expand_derive_from_row(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(feature = "migrate")]
#[proc_macro]
pub fn migrate(input: TokenStream) -> TokenStream {
    use syn::LitStr;

    let input = syn::parse_macro_input!(input as LitStr);
    match migrate::expand_migrator_from_lit_dir(input) {
        Ok(ts) => ts.into(),
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote!(::std::compile_error!(#msg)).into()
            }
        }
    }
}

#[proc_macro_attribute]
pub fn test(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as ArgsParser).0;
    let input = parse_macro_input!(input as ItemFn);

    test_attr::expand(args, input)
}
