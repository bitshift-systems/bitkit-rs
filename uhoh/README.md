# `uhoh`

`uhoh` derives consistent, user-facing application errors from an enum. Each
variant has a stable error code, a formatted message, and—when the Axum
feature is enabled—an HTTP status and `IntoResponse` implementation.

## Basic errors

Use a message as the first argument and a named `code` argument for every
variant:

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

The derive implements `Display` and `std::error::Error`, and adds:

- `code(&self) -> &'static str`
- `service_error(&self) -> uhoh::ServiceError` with the `serde` feature

Enable `serde` to use `service_error()` and its serializable response shape:

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

Mark the sole field of a tuple variant with `#[from]` to generate a standard
`From<SourceError>` implementation. This lets `?` convert a source error into
your user-facing error automatically:

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

`#[from]` is intentionally limited to a one-field tuple variant, so each
conversion has one clear source type and destination variant.

### Transparent error wrappers

Use `#[error(transparent)]` on a one-field tuple variant to preserve the
wrapped `uhoh` error's message, code, status, and serialized service-error
body. It composes naturally with `#[from]`:

```rust
#[derive(Debug, uhoh::Error)]
enum RequestError {
    #[error(transparent)]
    Repo(#[from] RepoError),
}
```

Status forwarding requires `http`; service-error body forwarding requires
`serde`. Without either feature, wrappers preserve the message and code.

## Presets

Presets supply a safe message and HTTP status while keeping the error code
explicit. Built-in presets are `ise`, `not_found`, and `unauthorized`:

```rust
#[derive(Debug, uhoh::Error)]
enum RequestError {
    #[error(ise, code = "R0005")]
    Unknown(#[from] anyhow::Error),
}
```

It expands to the message `"Something went wrong while processing your
request."` and `StatusCode::INTERNAL_SERVER_ERROR`. An explicit `status = ...`
overrides the preset status when needed.

`not_found` returns `404 Not Found` with `"The requested resource was not
found."`; `unauthorized` returns `401 Unauthorized` with `"You are not
authorized to perform that action."`.

## Axum errors

Enable the `axum` feature when the error is returned from an Axum handler:

```toml
[dependencies]
uhoh = { version = "0.1", features = ["axum"] }
```

The `axum` feature enables `http` and `serde` automatically. Enable just `http`
to use `status_code()` without Axum or serialization.

Give **every** variant a `status` argument (or a preset that supplies one). The derive adds
`status_code()` and implements Axum's `IntoResponse` using that status and a
JSON `ServiceError` body.

Use a status name such as `status = INTERNAL_SERVER_ERROR` or
`status = NOT_FOUND`; no status-code import is needed. Qualified paths such as
`status = StatusCode::NOT_FOUND` and other status expressions also work. Bare
identifiers always refer to `http::StatusCode` constants; qualify custom
constants with their module path (for example, `status = crate::MY_STATUS`).

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

An enum must either specify `status` for every variant or for none of them.
This prevents a response implementation with an ambiguous status code. Errors
without `status` work in HTTP-free services and do not require Axum.

## Formatting messages

Tuple variants use standard positional formatting such as `{0}`. Named-field
variants can refer to fields by name, such as `{model}` and `{field}`. A
message without placeholders does not expose its variant payload, which is
useful for keeping internal context out of user-facing errors.
