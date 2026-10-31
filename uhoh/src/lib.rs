//! `uhoh` generates application errors with codes and optional JSON and HTTP responses.
//!
//! An enum is a type with named alternatives.
//! Each alternative is a variant.
//! The [`Error`] derive generates code for your enum.
//! It implements [`std::fmt::Display`] and [`std::error::Error`].
//! It also adds a `code()` method.
//!
//! Add `#[derive(Debug, uhoh::Error)]` to your enum.
//! `Error` does not generate [`Debug`], so the example derives both.
//! For each ordinary variant, supply a message and a code in double quotes:
//!
//! ```
//! #[derive(Debug, uhoh::Error)]
//! enum JobError {
//!     #[error("Job {0} failed.", code = "J001")]
//!     Failed(String),
//!
//!     #[error("No job named {name} exists.", code = "J002")]
//!     Missing { name: String },
//! }
//!
//! let error = JobError::Failed("daily-report".into());
//! assert_eq!(error.to_string(), "Job daily-report failed.");
//! assert_eq!(error.code(), "J001");
//! let _: &dyn std::error::Error = &error;
//! ```
//!
//! Your application supplies the codes.
//! The derive accepts duplicate codes.
//! If callers use codes to identify errors, keep those codes stable.
//!
//! # Features
//!
//! A feature enables an optional part of the library.
//! By default, `uhoh` enables no optional features.
//! Serialization converts a value to a format such as JSON.
//! Deserialization converts that format back to a value.
//! Choose the features that your application needs:
//!
//! | Feature | What it adds |
//! | --- | --- |
//! | `serde` | Adds `ServiceError` and `service_error()`. Supports serialization and deserialization with Serde. |
//! | `http` | Makes `http` available through `uhoh`. Adds `status_code()` to enums with statuses. |
//! | `axum` | Enables `http` and `serde`. Makes `axum` available through `uhoh`. Implements `IntoResponse` for enums with statuses. |
//!
//! To enable `axum`, add this dependency to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! uhoh = { version = "0.1", features = ["axum"] }
//! ```
//!
//! # Formatting messages
//!
//! A placeholder marks where a message inserts a field value.
//! A tuple variant stores values in fields without names.
//! Tuple fields use positions such as `{0}`.
//! Named fields use placeholders such as `{name}`.
//! Formatting controls such as `{0:?}` work as in `format!`.
//!
//! If a tuple message contains placeholders, the derive passes all fields as arguments in field order.
//! The message must use every argument.
//! A message without braces can omit all field values.
//! Messages with named fields can use only the fields they need.
//!
//! The displayed message also becomes the message in the serialized response.
//! Include only data that you intend to expose to callers.
//! A fixed message keeps internal field values out of the response.
//!
//! # Converting errors
//!
//! Use a tuple variant with one field for the source error.
//! Put `#[from]` on that field to generate `From<Source>` for the enum.
//! The `?` operator can then convert the source error into your application error:
//!
//! ```
//! #[derive(Debug, uhoh::Error)]
//! enum ConfigError {
//!     #[error("Could not read configuration.", code = "C001")]
//!     Read(#[from] std::io::Error),
//! }
//!
//! fn read_config(path: &std::path::Path) -> Result<String, ConfigError> {
//!     Ok(std::fs::read_to_string(path)?)
//! }
//!
//! let error: ConfigError = std::io::Error::other("internal context").into();
//! assert_eq!(error.to_string(), "Could not read configuration.");
//! ```
//!
//! Error chaining links an error to its underlying cause.
//! `#[from]` does not implement error chaining.
//! The generated [`std::error::Error::source`] uses the default implementation and returns `None`.
//! This also applies to transparent wrappers, described below.
//!
//! # Transparent wrappers
//!
//! A transparent wrapper returns values from the error it contains.
//! `#[error(transparent)]` requires a tuple variant with one field.
//! It accepts no replacement message, code, or status.
//! It returns the displayed message and `code()` from the inner error.
//!
//! With `serde`, it also returns `service_error()` from the inner error.
//! With `http`, it also returns `status_code()` from the inner error.
//! The inner type must provide these methods for the enabled features.
//! A type that implements only `std::error::Error` does not meet these requirements.
//!
//! ```
//! #[derive(Debug, uhoh::Error)]
//! enum RepositoryError {
//!     #[error(not_found, code = "R404")]
//!     Missing,
//! }
//!
//! #[derive(Debug, uhoh::Error)]
//! enum RequestError {
//!     #[error(transparent)]
//!     Repository(#[from] RepositoryError),
//! }
//!
//! let error: RequestError = RepositoryError::Missing.into();
//! assert_eq!(error.code(), "R404");
//! assert_eq!(error.to_string(), "The requested resource was not found.");
//! ```
//!
//! With `http`, transparent variants count as variants with a status.
//! Every other variant in the enum must then supply a status.
//!
//! # HTTP statuses and presets
//!
//! Set `status = NOT_FOUND` to use a `http::StatusCode` constant without an import.
//! Paths and expressions also work, such as `status = uhoh::http::StatusCode::NOT_FOUND`.
//! A name without a path always refers to a `StatusCode` constant.
//! For application constants, include the module path, for example, `status = crate::MY_STATUS`.
//!
//! Every variant must supply a status, or all variants must omit it.
//! A preset supplies a fixed message and an HTTP status.
//! You must still supply an explicit code when you use a preset:
//!
//! | Preset | Message | Status |
//! | --- | --- | --- |
//! | `ise` | Something went wrong while processing your request. | 500 |
//! | `not_found` | The requested resource was not found. | 404 |
//! | `unauthorized` | You are not authorized to perform that action. | 401 |
//!
//! An explicit `status = ...` replaces the status from the preset.
//! The derive accepts `status` arguments and presets without the `http` feature.
//! Without `http`, these arguments do not generate `status_code()` or HTTP responses.
//!
//! With `axum`, enums with statuses implement `IntoResponse`.
//! The response uses the selected status and a JSON `ServiceError` body:
//!
//! ```
//! # #[cfg(feature = "axum")]
//! # {
//! use uhoh::axum::response::IntoResponse;
//!
//! #[derive(Debug, uhoh::Error)]
//! enum RequestError {
//!     #[error("Invalid value for {field}.", code = "R400", status = BAD_REQUEST)]
//!     Invalid { field: String },
//!
//!     #[error(ise, code = "R500")]
//!     Internal(std::io::Error),
//! }
//!
//! let error = RequestError::Invalid { field: "name".into() };
//! assert_eq!(error.status_code(), uhoh::http::StatusCode::BAD_REQUEST);
//! let response = error.into_response();
//! assert_eq!(response.status(), uhoh::http::StatusCode::BAD_REQUEST);
//! # }
//! ```
//!
//! # Serialized errors
//!
//! With `serde`, `service_error()` returns a `ServiceError`.
//! For ordinary variants, it contains the code, formatted message, and `details: None`.
//! Transparent variants return the entire `ServiceError` from the inner error, including its details.
//!
//! ```
//! # #[cfg(feature = "serde")]
//! # {
//! #[derive(Debug, uhoh::Error)]
//! enum JobError {
//!     #[error("Job {0} failed.", code = "J001")]
//!     Failed(String),
//! }
//!
//! let body = JobError::Failed("daily-report".into()).service_error();
//! assert_eq!(body.code, "J001");
//! assert_eq!(body.message, "Job daily-report failed.");
//! assert!(body.details.is_none());
//! assert_eq!(
//!     serde_json::to_string(&body).unwrap(),
//!     r#"{"code":"J001","message":"Job daily-report failed.","details":null}"#,
//! );
//! # }
//! ```

extern crate self as uhoh;

#[cfg(feature = "axum")]
/// The `axum` feature makes Axum available through `uhoh`.
pub use axum;
#[cfg(feature = "http")]
/// The `http` feature makes HTTP types available through `uhoh`.
pub use http;
pub use uhoh_macros::Error;

/// This type stores an application error body.
/// It is available with the `serde` feature.
///
/// Serialization converts a value to a format such as JSON.
/// For ordinary variants, generated `service_error()` methods set `code` and `message`.
/// They leave `details` as `None`, which Serde converts to JSON `null`.
/// You can set `details` on the returned value before serialization.
/// With `axum`, the generated `IntoResponse` implementation creates its own body through `service_error()`.
///
/// Deserialization converts a format such as JSON back to a value.
/// This type also implements `Deserialize` from Serde.
/// Its `&'static str` code borrows a string that must remain valid for the entire program.
/// It cannot deserialize arbitrary response buffers into an error that owns its code.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceError {
    /// This field stores the code that the application defines independently of the displayed message.
    pub code: &'static str,
    /// This field stores the message that `Display` produces for the caller.
    pub message: String,
    /// This field stores optional data about the error.
    /// Serde converts `None` to `null` and keeps the field in the JSON output.
    pub details: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[cfg(feature = "serde")]
    use super::ServiceError;

    #[test]
    #[cfg(feature = "serde")]
    fn test_service_error_serialization() {
        let error = ServiceError {
            code: "ERR001",
            message: "An error occurred".to_string(),
            details: Some(serde_json::json!({"info": "Additional details"})),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        let expected = r#"{"code":"ERR001","message":"An error occurred","details":{"info":"Additional details"}}"#;
        assert_eq!(serialized, expected);
    }

    #[cfg(feature = "axum")]
    use http::StatusCode;

    #[cfg(feature = "axum")]
    #[derive(Debug, Error)]
    enum DatabaseError {
        #[error("Connection failure", code = "D101", status = StatusCode::INTERNAL_SERVER_ERROR)]
        ConnectionError(String),

        #[error("Query failure: {0}", code = "D201", status = StatusCode::INTERNAL_SERVER_ERROR)]
        QueryError(String),

        #[error(
            "The {model} with {field} {value} was not found.",
            code = "D404",
            status = StatusCode::NOT_FOUND
        )]
        NotFound {
            model: String,
            field: String,
            value: String,
        },
    }

    #[cfg(feature = "axum")]
    #[test]
    fn derives_display_and_metadata() {
        let error = DatabaseError::NotFound {
            model: "user".into(),
            field: "id".into(),
            value: "42".into(),
        };

        assert_eq!(error.to_string(), "The user with id 42 was not found.");
        assert_eq!(error.code(), "D404");
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(error.service_error().details, None);
    }

    #[cfg(feature = "axum")]
    #[test]
    fn derives_axum_response() {
        use crate::axum::response::IntoResponse;
        let response = DatabaseError::QueryError("bad syntax".into()).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[derive(Debug, Error)]
    enum WorkerError {
        #[error("the job failed", code = "W001")]
        Failed,
    }

    #[test]
    fn supports_non_http_errors() {
        assert_eq!(WorkerError::Failed.code(), "W001");
    }

    #[derive(Debug)]
    struct FilterError;

    impl std::fmt::Display for FilterError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("invalid expression")
        }
    }

    #[derive(Debug, Error)]
    enum ConvertingError {
        #[error("invalid filter: {0}", code = "F001")]
        Filter(#[from] FilterError),
    }

    #[test]
    fn derives_from_for_marked_tuple_fields() {
        let error: ConvertingError = FilterError.into();

        assert_eq!(error.to_string(), "invalid filter: invalid expression");
    }

    #[cfg(feature = "axum")]
    #[derive(Debug, Error)]
    enum PresetError {
        #[error(ise, code = "R0005")]
        Unknown(#[from] FilterError),
    }

    #[cfg(feature = "axum")]
    #[test]
    fn applies_the_internal_server_error_preset() {
        let error: PresetError = FilterError.into();

        assert_eq!(
            error.to_string(),
            "Something went wrong while processing your request."
        );
        assert_eq!(error.code(), "R0005");
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[cfg(feature = "axum")]
    #[derive(Debug, Error)]
    enum OverriddenPresetError {
        #[error(ise, code = "R0400", status = StatusCode::BAD_REQUEST)]
        InvalidRequest,
    }

    #[cfg(feature = "axum")]
    #[test]
    fn lets_explicit_status_override_a_preset() {
        assert_eq!(
            OverriddenPresetError::InvalidRequest.status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[cfg(feature = "axum")]
    #[test]
    fn supports_unused_variant_fields() {
        let error = DatabaseError::ConnectionError("private context".into());

        assert_eq!(error.to_string(), "Connection failure");
    }
}
