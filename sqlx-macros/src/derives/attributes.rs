use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Attribute, DeriveInput, Field, Lit, Meta, MetaNameValue, Variant, Expr, LitStr, Token, Path};

macro_rules! assert_attribute {
    ($e:expr, $err:expr, $input:expr) => {
        if !$e {
            return Err(syn::Error::new_spanned($input, $err));
        }
    };
}

macro_rules! fail {
    ($t:expr, $m:expr) => {
        return Err(syn::Error::new_spanned($t, $m))
    };
}

macro_rules! try_set {
    ($i:ident, $v:expr, $t:expr) => {
        match $i {
            None => $i = Some($v),
            Some(_) => fail!($t, "duplicate attribute"),
        }
    };
}

pub struct TypeName {
    pub val: String,
    pub span: Span,
    /// Whether the old sqlx(rename) syntax was used instead of sqlx(type_name)
    pub deprecated_rename: bool,
}

impl TypeName {
    pub fn get(&self) -> TokenStream {
        let val = &self.val;
        if self.deprecated_rename {
            quote_spanned!(self.span=> {
                ::sqlx_oldapi::_rename();
                #val
            })
        } else {
            quote! { #val }
        }
    }
}

#[derive(Copy, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum RenameAll {
    LowerCase,
    SnakeCase,
    UpperCase,
    ScreamingSnakeCase,
    KebabCase,
    CamelCase,
    PascalCase,
}

pub struct SqlxContainerAttributes {
    pub transparent: bool,
    pub type_name: Option<TypeName>,
    pub rename_all: Option<RenameAll>,
    pub repr: Option<Ident>,
}

pub struct SqlxChildAttributes {
    pub rename: Option<String>,
    pub default: bool,
    pub flatten: bool,
    pub try_from: Option<Ident>,
}

pub fn parse_container_attributes(input: &[Attribute]) -> syn::Result<SqlxContainerAttributes> {
    let mut transparent = None;
    let mut repr = None;
    let mut type_name = None;
    let mut rename_all = None;

    for attr in input
        .iter()
        .filter(|a| a.path().is_ident("sqlx") || a.path().is_ident("repr"))
    {
        match &attr.meta {
            Meta::List(list) if list.path.is_ident("sqlx") => {
                let nested_metas = list.parse_args_with(Punctuated::<Meta, syn::token::Comma>::parse_terminated)?;
                for meta_item in nested_metas {
                    match meta_item {
                        Meta::Path(p) if p.is_ident("transparent") => {
                            try_set!(transparent, true, p)
                        }
                        Meta::NameValue(mnv) if mnv.path.is_ident("rename_all") => {
                            if let Expr::Lit(expr_lit) = &mnv.value {
                                if let Lit::Str(val_str) = &expr_lit.lit {
                                    let val = match &*val_str.value() {
                                        "lowercase" => RenameAll::LowerCase,
                                        "snake_case" => RenameAll::SnakeCase,
                                        "UPPERCASE" => RenameAll::UpperCase,
                                        "SCREAMING_SNAKE_CASE" => RenameAll::ScreamingSnakeCase,
                                        "kebab-case" => RenameAll::KebabCase,
                                        "camelCase" => RenameAll::CamelCase,
                                        "PascalCase" => RenameAll::PascalCase,
                                        _ => fail!(val_str, "unexpected value for rename_all"),
                                    };
                                    try_set!(rename_all, val, &mnv.path)
                                } else {
                                    fail!(expr_lit, "expected string literal for rename_all")
                                }
                            } else {
                                fail!(&mnv.value, "expected literal expression for rename_all")
                            }
                        }
                        Meta::NameValue(mnv) if mnv.path.is_ident("type_name") => {
                            if let Expr::Lit(expr_lit) = &mnv.value {
                                if let Lit::Str(val_str) = &expr_lit.lit {
                                    try_set!(
                                        type_name,
                                        TypeName {
                                            val: val_str.value(),
                                            span: val_str.span(),
                                            deprecated_rename: false
                                        },
                                        &mnv.path
                                    )
                                } else {
                                    fail!(expr_lit, "expected string literal for type_name")
                                }
                            } else {
                                fail!(&mnv.value, "expected literal expression for type_name")
                            }
                        }
                        Meta::NameValue(mnv) if mnv.path.is_ident("rename") => {
                            if let Expr::Lit(expr_lit) = &mnv.value {
                                if let Lit::Str(val_str) = &expr_lit.lit {
                                    try_set!(
                                        type_name,
                                        TypeName {
                                            val: val_str.value(),
                                            span: val_str.span(),
                                            deprecated_rename: true
                                        },
                                        &mnv.path
                                    )
                                } else {
                                    fail!(expr_lit, "expected string literal for rename")
                                }
                            } else {
                                fail!(&mnv.value, "expected literal expression for rename")
                            }
                        }
                        u => fail!(u, "unexpected attribute inside sqlx(...)"),
                    }
                }
            }
            Meta::List(list) if list.path.is_ident("repr") => {
                let nested_metas = list.parse_args_with(Punctuated::<Meta, syn::token::Comma>::parse_terminated)?;
                if nested_metas.len() != 1 {
                    fail!(&list.path, "expected one value for repr")
                }
                match nested_metas.first().unwrap() {
                    Meta::Path(p) if p.get_ident().is_some() => {
                        try_set!(repr, p.get_ident().unwrap().clone(), &list.path);
                    }
                    u => fail!(u, "unexpected value for repr"),
                }
            }
            _ => { /* Not an attribute we are interested in, or not a list */ }
        }
    }

    Ok(SqlxContainerAttributes {
        transparent: transparent.unwrap_or(false),
        repr,
        type_name,
        rename_all,
    })
}

pub fn parse_child_attributes(input: &[Attribute]) -> syn::Result<SqlxChildAttributes> {
    let mut rename = None;
    let mut default = false;
    let mut try_from = None;
    let mut flatten = false;

    for attr in input.iter().filter(|a| a.path().is_ident("sqlx")) {
        if let Meta::List(list) = &attr.meta {
            let nested_metas = list.parse_args_with(Punctuated::<Meta, syn::token::Comma>::parse_terminated)?;
            for meta_item in nested_metas {
                match meta_item {
                    Meta::NameValue(mnv) if mnv.path.is_ident("rename") => {
                        if let Expr::Lit(expr_lit) = &mnv.value {
                            if let Lit::Str(val_str) = &expr_lit.lit {
                                try_set!(rename, val_str.value(), &mnv.path)
                            } else {
                                fail!(expr_lit, "expected string literal for rename")
                            }
                        } else {
                            fail!(&mnv.value, "expected literal expression for rename")
                        }
                    }
                    Meta::NameValue(mnv) if mnv.path.is_ident("try_from") => {
                        if let Expr::Lit(expr_lit) = &mnv.value {
                            if let Lit::Str(val_str) = &expr_lit.lit {
                                try_set!(try_from, val_str.parse()?, &mnv.path)
                            } else {
                                fail!(expr_lit, "expected string literal for try_from")
                            }
                        } else {
                            fail!(&mnv.value, "expected literal expression for try_from")
                        }
                    }
                    Meta::Path(path) if path.is_ident("default") => default = true,
                    Meta::Path(path) if path.is_ident("flatten") => flatten = true,
                    u => fail!(u, "unexpected attribute inside sqlx(...)"),
                }
            }
        }
    }

    Ok(SqlxChildAttributes {
        rename,
        default,
        flatten,
        try_from,
    })
}

pub fn check_transparent_attributes(
    input: &DeriveInput,
    field: &Field,
) -> syn::Result<SqlxContainerAttributes> {
    let attributes = parse_container_attributes(&input.attrs)?;

    assert_attribute!(
        attributes.rename_all.is_none(),
        "unexpected #[sqlx(rename_all = ..)]",
        field
    );

    let ch_attributes = parse_child_attributes(&field.attrs)?;

    assert_attribute!(
        ch_attributes.rename.is_none(),
        "unexpected #[sqlx(rename = ..)]",
        field
    );

    Ok(attributes)
}

pub fn check_enum_attributes(input: &DeriveInput) -> syn::Result<SqlxContainerAttributes> {
    let attributes = parse_container_attributes(&input.attrs)?;

    assert_attribute!(
        !attributes.transparent,
        "unexpected #[sqlx(transparent)]",
        input
    );

    Ok(attributes)
}

pub fn check_weak_enum_attributes(
    input: &DeriveInput,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<SqlxContainerAttributes> {
    let attributes = check_enum_attributes(input)?;

    assert_attribute!(attributes.repr.is_some(), "expected #[repr(..)]", input);

    assert_attribute!(
        attributes.rename_all.is_none(),
        "unexpected #[sqlx(c = ..)]",
        input
    );

    for variant in variants {
        let attributes = parse_child_attributes(&variant.attrs)?;

        assert_attribute!(
            attributes.rename.is_none(),
            "unexpected #[sqlx(rename = ..)]",
            variant
        );
    }

    Ok(attributes)
}

pub fn check_strong_enum_attributes(
    input: &DeriveInput,
    _variants: &Punctuated<Variant, Comma>,
) -> syn::Result<SqlxContainerAttributes> {
    let attributes = check_enum_attributes(input)?;

    assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", input);

    Ok(attributes)
}

pub fn check_struct_attributes(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<SqlxContainerAttributes> {
    let attributes = parse_container_attributes(&input.attrs)?;

    assert_attribute!(
        !attributes.transparent,
        "unexpected #[sqlx(transparent)]",
        input
    );

    assert_attribute!(
        attributes.rename_all.is_none(),
        "unexpected #[sqlx(rename_all = ..)]",
        input
    );

    assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", input);

    for field in fields {
        let attributes = parse_child_attributes(&field.attrs)?;

        assert_attribute!(
            attributes.rename.is_none(),
            "unexpected #[sqlx(rename = ..)]",
            field
        );
    }

    Ok(attributes)
}
