# `uhoh`

`uhoh` generates application errors with messages and codes from an enum.
An enum is a type with named alternatives.
Each alternative is a variant.

[![Crates.io](https://img.shields.io/crates/v/uhoh)](https://crates.io/crates/uhoh)
[![Documentation](https://docs.rs/uhoh/badge.svg)][docs]

Read the [API reference][docs] for more examples and details.

## Basic errors

The `Error` derive generates code for your enum.
Use a message as the first argument for each variant.
Give each variant a named `code` argument:

```rust
#[derive(Debug, uhoh::Error)]
enum WorkerError {
    #[error("The worker could not start.", code = "W101")]
    Start,

    #[error("Job {0} failed.", code = "W201")]
    JobFailed(String),
}

let error = WorkerError::JobFailed("daily-report".into());
assert_eq!(error.to_string(), "Job daily-report failed.");
assert_eq!(error.code(), "W201");
```

The derive implements `Display` and `std::error::Error`.
It also adds `code(&self) -> &'static str`.

A feature enables an optional part of the library.
With the `serde` feature, it adds `service_error(&self) -> uhoh::ServiceError`.

Serialization converts a value to a format such as JSON.
To enable `service_error()` and serialization, add the `serde` feature to your
`Cargo.toml` dependency:

```toml
[dependencies]
uhoh = { path = "../uhoh", features = ["serde"] }
```

```json
{
  "code": "W201",
  "message": "Job daily-report failed.",
  "details": null
}
```

### Converting source errors

A tuple variant stores values in fields without names.
Use a tuple variant with one field for the source error.
Mark that field with `#[from]` to generate `From<SourceError>`.
The `?` operator can then convert the source error into your application error:

```rust
#[derive(Debug, uhoh::Error)]
enum DatabaseError {
    #[error("invalid filter: {0}", code = "D0400")]
    Filter(#[from] FilterError),
}

fn load_filter() -> Result<(), DatabaseError> {
    let filter = parse_filter()?; // `FilterError` converts to `DatabaseError`
    Ok(())
}
```

`#[from]` supports only tuple variants with one field.
Each conversion has one source type and one destination variant.

### Transparent error wrappers

A transparent wrapper returns values from the error it contains.
Use `#[error(transparent)]` on a tuple variant with one field.
The variant preserves the message, code, status, and serialized response body
of the inner `uhoh` error.
You can combine it with `#[from]`:

```rust
#[derive(Debug, uhoh::Error)]
enum RequestError {
    #[error(transparent)]
    Repo(#[from] RepoError),
}
```

To return the inner status, enable `http`.
To return the inner response body, enable `serde`.
Without either feature, wrappers preserve the message and code.

## Presets

A preset supplies a fixed message and an HTTP status.
The message does not expose the data that the error stores.
You must still supply the error code.
The built-in presets are `ise`, `not_found`, and `unauthorized`.
This example uses `ise`:

```rust
#[derive(Debug, uhoh::Error)]
enum RequestError {
    #[error(ise, code = "R0005")]
    Unknown(#[from] anyhow::Error),
}
```

The `ise` preset supplies `"Something went wrong while processing your request."`
and `StatusCode::INTERNAL_SERVER_ERROR`.
An explicit `status = ...` replaces the status from the preset.

The `not_found` preset returns `404 Not Found` with `"The requested resource was not found."`.
The `unauthorized` preset returns `401 Unauthorized` with `"You are not authorized to perform that action."`.

## Axum errors

An Axum handler is a function that handles a request.
If your Axum handler returns the error, enable the `axum` feature in your
`Cargo.toml` dependency:

```toml
[dependencies]
uhoh = { version = "0.1", features = ["axum"] }
```

The `axum` feature enables `http` and `serde` automatically.
To use `status_code()` without Axum or serialization, enable only `http`.

Give every variant a `status` argument or a preset that supplies one.
The derive adds `status_code()` and implements `IntoResponse` for Axum.
The response uses that status and a JSON `ServiceError` body.

Use a status name such as `status = INTERNAL_SERVER_ERROR` or `status = NOT_FOUND`.
These names need no import.
Paths such as `status = StatusCode::NOT_FOUND` and other status expressions also work.
A name without a path always refers to a `http::StatusCode` constant.
For custom constants, include the module path, for example, `status = crate::MY_STATUS`.

```rust
use uhoh::axum::{
    http::StatusCode,
    response::IntoResponse,
};

#[derive(Debug, uhoh::Error)]
enum DatabaseError {
    #[error(
        "Connection failure",
        code = "D101",
        status = INTERNAL_SERVER_ERROR
    )]
    Connection(String),

    #[error(
        "The {model} with {field} {value} was not found.",
        code = "D404",
        status = NOT_FOUND
    )]
    NotFound {
        model: String,
        field: String,
        value: String,
    },
}

let error = DatabaseError::NotFound {
    model: "user".into(),
    field: "id".into(),
    value: "42".into(),
};
assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
let response = error.into_response();
assert_eq!(response.status(), StatusCode::NOT_FOUND);
```

An enum must supply a status for every variant or for none of them.
This rule gives each response a defined status code.
Errors without a status work in services that do not use HTTP or Axum.

## Formatting messages

A placeholder marks where a message inserts a field value.
Tuple variants refer to fields by position, such as `{0}`.
Variants with named fields can use names, such as `{model}` and `{field}`.
A message without placeholders does not expose the data that the variant stores.
This keeps internal data out of messages that users see.

[docs]: https://docs.rs/uhoh
