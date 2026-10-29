//! Derive macro implementation for `uhoh`.
//!
//! Applications should use the `Error` re-export from `uhoh`.

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod expand;

/// Derives application-error formatting, codes, and feature-dependent responses.
///
/// Use on an enum that also implements `Debug`. Each variant must have one of:
///
/// - `#[error("message", code = "CODE", status = NOT_FOUND)]`, with optional status;
/// - `#[error(ise, code = "CODE")]` (also `not_found` or `unauthorized`);
/// - `#[error(transparent)]` on a one-field tuple variant, forwarding metadata.
///
/// The derive implements `Display` and `std::error::Error`, and adds
/// `code(&self) -> &'static str`. The `serde` feature adds `service_error()`.
/// With `http`, enums whose variants all have statuses gain `status_code()`;
/// with `axum`, those enums also implement `IntoResponse`.
///
/// Mark the sole field of a tuple variant with `#[from]` to generate a `From`
/// conversion. Neither conversion nor transparency forwards `Error::source()`.
///
/// See the [crate documentation](https://docs.rs/uhoh) for examples and the
/// complete feature and attribute guide.
#[proc_macro_derive(Error, attributes(error, from))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input).into()
}
