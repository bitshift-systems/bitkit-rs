//! This library generates code for `uhoh`.
//!
//! In applications, use `Error` through `uhoh`.

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod expand;

/// This derive generates code to format application errors and return error codes and responses.
///
/// An enum is a type with named alternatives.
/// Each alternative is a variant.
/// Use this derive on an enum that also implements `Debug`.
///
/// A preset supplies a fixed message and an HTTP status.
/// A tuple variant stores values in fields without names.
/// A transparent wrapper returns values from the error it contains.
/// Each variant must use one of these forms:
///
/// - Supply a message and code: `#[error("message", code = "CODE", status = NOT_FOUND)]`. The status is optional.
/// - Use a preset: `#[error(ise, code = "CODE")]`. The other presets are `not_found` and `unauthorized`.
/// - Return values from an inner error: `#[error(transparent)]`. This requires a tuple variant with one field.
///
/// The derive implements `Display` and `std::error::Error`.
/// It adds `code(&self) -> &'static str`.
///
/// A feature enables an optional part of the library.
/// The `serde` feature adds `service_error()`.
/// With `http`, enums whose variants all have statuses also have `status_code()`.
/// With `axum`, those enums also implement `IntoResponse`.
///
/// For a tuple variant with one field, mark that field with `#[from]` to generate a `From` conversion.
/// This lets Rust convert the field value into your application error.
/// Neither conversion nor transparent wrappers return the inner `Error::source()`.
///
/// Read the [API reference](https://docs.rs/uhoh) for examples and details about features and attributes.
#[proc_macro_derive(Error, attributes(error, from))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input).into()
}
