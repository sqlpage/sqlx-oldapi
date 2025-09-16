use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{LitStr, Token};

#[allow(dead_code)]
struct Args {
    fixtures: Vec<LitStr>,
    migrations: MigrationsOpt,
}

#[allow(dead_code)]
enum MigrationsOpt {
    InferredPath,
    ExplicitPath(LitStr),
    ExplicitMigrator(syn::Path),
    Disabled,
}

pub fn expand(
    args: Punctuated<syn::Meta, Token![,]>,
    input: syn::ItemFn,
) -> proc_macro::TokenStream {
    let result: crate::Result<TokenStream> = if input.sig.inputs.is_empty() {
        if !args.is_empty() {
            if cfg!(feature = "migrate") {
                Err(syn::Error::new_spanned(
                    args.first().unwrap(),
                    "control attributes are not allowed unless \
                        the `migrate` feature is enabled and \
                        automatic test DB management is used; see docs",
                )
                .into())
            } else {
                Err(syn::Error::new_spanned(
                    args.first().unwrap(),
                    "control attributes are not allowed unless \
                    automatic test DB management is used; see docs",
                )
                .into())
            }
        } else {
            expand_simple(input)
        }
    } else {
        #[cfg(feature = "migrate")]
        {
            expand_advanced(args, input)
        }

        #[cfg(not(feature = "migrate"))]
        {
            Err(syn::Error::new_spanned(&input, "`migrate` feature required").into())
        }
    };

    match result {
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

fn expand_simple(input: syn::ItemFn) -> crate::Result<TokenStream> {
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    Ok(quote! {
        #[::core::prelude::v1::test]
        #(#attrs)*
        fn #name() #ret {
            ::sqlx_oldapi::test_block_on(async { #body })
        }
    })
}

#[cfg(feature = "migrate")]
fn expand_advanced(
    args: Punctuated<syn::Meta, Token![,]>,
    input: syn::ItemFn,
) -> crate::Result<TokenStream> {
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let body = &input.block;
    let attrs = &input.attrs;

    let args = parse_args(args)?;

    let fn_arg_types = inputs.iter().map(|_| quote! { _ });

    let fixtures = args.fixtures.into_iter().map(|fixture| {
        let path = format!("fixtures/{}.sql", fixture.value());

        quote! {
            ::sqlx_oldapi::testing::TestFixture {
                path: #path,
                contents: include_str!(#path),
            }
        }
    });

    let migrations = match args.migrations {
        MigrationsOpt::ExplicitPath(path) => {
            let migrator = crate::migrate::expand_migrator_from_lit_dir(path)?;
            quote! { args.migrator(&#migrator); }
        }
        MigrationsOpt::InferredPath if !inputs.is_empty() => {
            let migrations_path = crate::common::resolve_path("./migrations", input.sig.span())?;

            if migrations_path.is_dir() {
                let migrator = crate::migrate::expand_migrator(&migrations_path)?;
                quote! { args.migrator(&#migrator); }
            } else {
                quote! {}
            }
        }
        MigrationsOpt::ExplicitMigrator(path) => {
            quote! { args.migrator(&#path); }
        }
        _ => quote! {},
    };

    Ok(quote! {
        #[::core::prelude::v1::test]
        #(#attrs)*
        fn #name() #ret {
            async fn inner(#inputs) #ret {
                #body
            }

            let mut args = ::sqlx_oldapi::testing::TestArgs::new(concat!(module_path!(), "::", stringify!(#name)));

            #migrations

            args.fixtures(&[#(#fixtures),*]);

            // We need to give a coercion site or else we get "unimplemented trait" errors.
            let f: fn(#(#fn_arg_types),*) -> _ = inner;

            ::sqlx_oldapi::testing::TestFn::run_test(f, args)
        }
    })
}

#[cfg(feature = "migrate")]
fn parse_args(attr_args: Punctuated<syn::Meta, Token![,]>) -> syn::Result<Args> {
    let mut fixtures = vec![];
    let mut migrations = MigrationsOpt::InferredPath;

    for arg in attr_args {
        match arg {
            syn::Meta::List(list) if list.path.is_ident("fixtures") => {
                if !fixtures.is_empty() {
                    return Err(syn::Error::new_spanned(list, "duplicate `fixtures` arg"));
                }

                let parsed_fixtures = list.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
                for litstr in parsed_fixtures {
                    fixtures.push(litstr);
                }
            }
            syn::Meta::NameValue(namevalue)
                if namevalue.path.is_ident("migrations") =>
            {
                if !matches!(migrations, MigrationsOpt::InferredPath) {
                    return Err(syn::Error::new_spanned(
                        namevalue,
                        "cannot have more than one `migrations` or `migrator` arg",
                    ));
                }

                migrations = match &namevalue.value {
                    syn::Expr::Lit(ref expr_lit) => match &expr_lit.lit {
                        syn::Lit::Bool(litbool) => {
                            if !litbool.value {
                                // migrations = false
                                MigrationsOpt::Disabled
                            } else {
                                // migrations = true
                                return Err(syn::Error::new_spanned(
                                    expr_lit,
                                    "`migrations = true` is redundant",
                                ));
                            }
                        }
                        // migrations = "<path>"
                        syn::Lit::Str(litstr) => MigrationsOpt::ExplicitPath(litstr.clone()),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &namevalue.value,
                                "expected string or `false` for migrations value",
                            ))
                        }
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &namevalue.value,
                            "expected literal for migrations value",
                        ))
                    }
                };
            }
            syn::Meta::NameValue(namevalue)
                if namevalue.path.is_ident("migrator") =>
                {
                    if !matches!(migrations, MigrationsOpt::InferredPath) {
                        return Err(syn::Error::new_spanned(
                            namevalue,
                            "cannot have more than one `migrations` or `migrator` arg",
                        ));
                    }

                    migrations = match &namevalue.value {
                        // migrator = "<path>"
                        syn::Expr::Lit(ref expr_lit) => match &expr_lit.lit {
                            syn::Lit::Str(litstr) => MigrationsOpt::ExplicitMigrator(litstr.parse()?),
                             _ => {
                                return Err(syn::Error::new_spanned(
                                    &namevalue.value,
                                    "expected string for migrator path",
                                ))
                            }
                        },
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &namevalue.value,
                                "expected string literal for migrator path",
                            ))
                        }
                    };
                }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "expected `fixtures(\"<filename>\", ...)` or `migrations = \"<path>\" | false` or `migrator = \"<rust path>\"`"
                ))
            }
        }
    }

    Ok(Args {
        fixtures,
        migrations,
    })
}
