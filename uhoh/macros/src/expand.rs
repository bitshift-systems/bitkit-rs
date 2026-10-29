use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use serde_tokenstream::{ParseWrapper, from_tokenstream_spanned};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Fields, Lit, LitStr, Result, Token, Type,
    punctuated::Punctuated,
};

struct VariantError {
    code: TokenStream,
    status: Option<TokenStream>,
    display_pattern: TokenStream,
    code_pattern: TokenStream,
    status_pattern: TokenStream,
    service_error_pattern: TokenStream,
    message: TokenStream,
    transparent: bool,
}

#[derive(Deserialize)]
struct ErrorOptions {
    code: ParseWrapper<LitStr>,
    #[serde(default)]
    status: Option<ParseWrapper<Expr>>,
}

enum ErrorPreset {
    InternalServerError,
    NotFound,
    Unauthorized,
}

impl ErrorPreset {
    fn message(&self) -> &'static str {
        match self {
            Self::InternalServerError => "Something went wrong while processing your request.",
            Self::NotFound => "The requested resource was not found.",
            Self::Unauthorized => "You are not authorized to perform that action.",
        }
    }

    fn status(&self) -> TokenStream {
        match self {
            Self::InternalServerError => quote! {
                ::uhoh::http::StatusCode::INTERNAL_SERVER_ERROR
            },
            Self::NotFound => quote! {
                ::uhoh::http::StatusCode::NOT_FOUND
            },
            Self::Unauthorized => quote! {
                ::uhoh::http::StatusCode::UNAUTHORIZED
            },
        }
    }
}

struct FromVariant {
    source: Type,
    constructor: TokenStream,
}

pub fn derive(input: &DeriveInput) -> TokenStream {
    derive_inner(input).unwrap_or_else(Error::into_compile_error)
}

fn derive_inner(input: &DeriveInput) -> Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "`Error` can only be derived for enums",
        ));
    };

    let variants = data
        .variants
        .iter()
        .map(parse_variant)
        .collect::<Result<Vec<_>>>()?;
    let from_variants = data
        .variants
        .iter()
        .filter_map(|variant| from_variant(variant).transpose())
        .collect::<Result<Vec<_>>>()?;
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let display_arms = variants.iter().map(|variant| {
        let pattern = &variant.display_pattern;
        let message = &variant.message;
        quote! { #pattern => #message }
    });

    let code_arms = variants.iter().map(|variant| {
        let pattern = &variant.code_pattern;
        let code = &variant.code;
        quote! { #pattern => #code }
    });

    let status_arms = variants.iter().map(|variant| {
        let pattern = &variant.status_pattern;
        let status = variant.status.as_ref().expect("checked below");
        quote! { #pattern => #status }
    });
    let service_error_arms = variants.iter().map(|variant| {
        let pattern = &variant.service_error_pattern;
        if variant.transparent {
            quote! { #pattern => source.service_error() }
        } else {
            quote! {
                #pattern => ::uhoh::ServiceError {
                    code: self.code(),
                    message: self.to_string(),
                    details: None,
                }
            }
        }
    });

    let has_status = variants.iter().any(|variant| variant.status.is_some());

    if has_status && variants.iter().any(|variant| variant.status.is_none()) {
        return Err(Error::new_spanned(
            input,
            "either every variant must specify `status = ...`, or none may specify it",
        ));
    }

    let http_impl = (cfg!(feature = "http") && has_status).then(|| {
        quote! {
            pub fn status_code(&self) -> ::uhoh::http::StatusCode {
                match self { #(#status_arms),* }
            }
        }
    });

    let into_response_impl = (cfg!(feature = "axum") && has_status).then(|| {
        quote! {
            impl #impl_generics ::uhoh::axum::response::IntoResponse for #name #type_generics #where_clause {
                fn into_response(self) -> ::uhoh::axum::response::Response {
                    use ::uhoh::axum::response::IntoResponse as _;

                    let status = self.status_code();
                    (status, ::uhoh::axum::Json(self.service_error())).into_response()
                }
            }
        }
    });

    let service_error_impl = cfg!(feature = "serde").then(|| {
        quote! {
            pub fn service_error(&self) -> ::uhoh::ServiceError {
                match self { #(#service_error_arms),* }
            }
        }
    });

    let from_impls = from_variants.iter().map(|variant| {
        let source = &variant.source;
        let constructor = &variant.constructor;

        quote! {
            impl #impl_generics ::core::convert::From<#source> for #name #type_generics #where_clause {
                fn from(source: #source) -> Self {
                    #constructor
                }
            }
        }
    });

    Ok(quote! {
        impl #impl_generics ::core::fmt::Display for #name #type_generics #where_clause {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let message = match self { #(#display_arms),* };
                formatter.write_str(&message)
            }
        }

        impl #impl_generics ::std::error::Error for #name #type_generics #where_clause {}

        impl #impl_generics #name #type_generics #where_clause {
            pub fn code(&self) -> &'static str {
                match self { #(#code_arms),* }
            }

            #http_impl

            #service_error_impl
        }

        #into_response_impl

        #(#from_impls)*
    })
}

fn from_variant(variant: &syn::Variant) -> Result<Option<FromVariant>> {
    let Fields::Unnamed(fields) = &variant.fields else {
        if variant.fields.iter().any(|field| {
            field
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("from"))
        }) {
            return Err(Error::new_spanned(
                variant,
                "`#[from]` is only supported on the sole field of a tuple variant",
            ));
        }
        return Ok(None);
    };

    let marked_fields = fields
        .unnamed
        .iter()
        .filter(|field| {
            field
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("from"))
        })
        .collect::<Vec<_>>();
    if marked_fields.is_empty() {
        return Ok(None);
    }
    if marked_fields.len() != 1 || fields.unnamed.len() != 1 {
        return Err(Error::new_spanned(
            variant,
            "`#[from]` is only supported on the sole field of a tuple variant",
        ));
    }

    let source = marked_fields[0].ty.clone();
    let ident = &variant.ident;
    Ok(Some(FromVariant {
        source,
        constructor: quote! { Self::#ident(source) },
    }))
}

fn parse_variant(variant: &syn::Variant) -> Result<VariantError> {
    let attribute = variant
        .attrs
        .iter()
        .find(|attribute| attribute.path().is_ident("error"))
        .ok_or_else(|| {
            Error::new_spanned(
                variant,
                "every variant needs `#[error(message, code = \"...\")]`",
            )
        })?;
    if is_transparent(attribute)? {
        return transparent_variant(variant);
    }

    let (template, code, status) = parse_attribute(attribute)?;
    let ident = variant.ident.clone();

    let (display_pattern, metadata_pattern, message) = match &variant.fields {
        Fields::Unit => (
            quote! { Self::#ident },
            quote! { Self::#ident },
            quote! { ::std::format!(#template) },
        ),
        Fields::Unnamed(fields) => {
            let bindings = (0..fields.unnamed.len())
                .map(|index| format_ident!("field_{index}"))
                .collect::<Vec<_>>();
            let message = if template.value().contains('{') {
                quote! { ::std::format!(#template, #(#bindings),*) }
            } else {
                quote! { ::std::format!(#template) }
            };
            (
                quote! { Self::#ident(#(#bindings),*) },
                quote! { Self::#ident(..) },
                message,
            )
        }
        Fields::Named(fields) => {
            let bindings = fields
                .named
                .iter()
                .map(|field| field.ident.clone().expect("named field"))
                .collect::<Vec<_>>();
            (
                quote! { Self::#ident { #(#bindings),* } },
                quote! { Self::#ident { .. } },
                quote! { ::std::format!(#template) },
            )
        }
    };

    Ok(VariantError {
        code: quote! { #code },
        status,
        display_pattern,
        code_pattern: metadata_pattern.clone(),
        status_pattern: metadata_pattern.clone(),
        service_error_pattern: metadata_pattern,
        message,
        transparent: false,
    })
}

fn is_transparent(attribute: &Attribute) -> Result<bool> {
    let args = attribute.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
    Ok(args.len() == 1
        && matches!(args.first(), Some(Expr::Path(path)) if path.path.is_ident("transparent")))
}

fn transparent_variant(variant: &syn::Variant) -> Result<VariantError> {
    let Fields::Unnamed(fields) = &variant.fields else {
        return Err(Error::new_spanned(
            variant,
            "`#[error(transparent)]` requires a one-field tuple variant",
        ));
    };
    if fields.unnamed.len() != 1 {
        return Err(Error::new_spanned(
            variant,
            "`#[error(transparent)]` requires a one-field tuple variant",
        ));
    }

    let ident = &variant.ident;
    let pattern = quote! { Self::#ident(source) };
    Ok(VariantError {
        code: quote! { source.code() },
        status: cfg!(feature = "http").then(|| quote! { source.status_code() }),
        display_pattern: pattern.clone(),
        code_pattern: pattern.clone(),
        status_pattern: pattern.clone(),
        service_error_pattern: pattern,
        message: quote! { source.to_string() },
        transparent: true,
    })
}

fn parse_attribute(attribute: &Attribute) -> Result<(LitStr, LitStr, Option<TokenStream>)> {
    let args = attribute.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
    let (template, preset) = match args.first() {
        Some(Expr::Lit(message)) => match &message.lit {
            Lit::Str(template) => (template.clone(), None),
            _ => {
                return Err(Error::new_spanned(
                    message,
                    "the first error argument must be a message string literal or a preset",
                ));
            }
        },
        Some(Expr::Path(path)) if path.path.is_ident("ise") => {
            let preset = ErrorPreset::InternalServerError;
            (
                LitStr::new(preset.message(), path.path.segments[0].ident.span()),
                Some(preset),
            )
        }
        Some(Expr::Path(path)) if path.path.is_ident("not_found") => {
            let preset = ErrorPreset::NotFound;
            (
                LitStr::new(preset.message(), path.path.segments[0].ident.span()),
                Some(preset),
            )
        }
        Some(Expr::Path(path)) if path.path.is_ident("unauthorized") => {
            let preset = ErrorPreset::Unauthorized;
            (
                LitStr::new(preset.message(), path.path.segments[0].ident.span()),
                Some(preset),
            )
        }
        Some(expression) => {
            return Err(Error::new_spanned(
                expression,
                "unknown error preset; expected a message string literal or `ise`",
            ));
        }
        _ => {
            return Err(Error::new_spanned(
                attribute,
                "the first error argument must be a message string literal or a preset",
            ));
        }
    };

    let options_tokens = args.into_iter().skip(1).collect::<Vec<_>>();
    let options_tokens = quote! { #(#options_tokens),* };
    let delimiter_span = attribute.meta.require_list()?.delimiter.span();
    let options = from_tokenstream_spanned::<ErrorOptions>(delimiter_span, &options_tokens)?;

    Ok((
        template,
        options.code.into_inner(),
        options
            .status
            .map(|status| {
                let expression = status.into_inner();
                if let Expr::Path(path) = &expression
                    && path.qself.is_none()
                    && let Some(name) = path.path.get_ident()
                {
                    quote! { ::uhoh::http::StatusCode::#name }
                } else {
                    expression.into_token_stream()
                }
            })
            .or_else(|| preset.map(|preset| preset.status())),
    ))
}
