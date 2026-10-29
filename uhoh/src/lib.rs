//! Application errors with explicit codes and optional JSON and HTTP responses.
//!
//! Derive [`Error`] on an enum to implement [`std::fmt::Display`] and
//! [`std::error::Error`], and to add a `code()` method. Derive [`Debug`] separately.
//! Each ordinary variant needs a message and a string-literal code:
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
//! Codes are supplied by your application; the derive does not check their
//! uniqueness. Keep them stable when callers use them to identify errors.
//!
//! # Features
//!
//! No optional features are enabled by default.
//!
//! | Feature | Additional functionality |
//! | --- | --- |
//! | `serde` | `ServiceError` and `service_error()`, with Serde serialization and deserialization. |
//! | `http` | Re-exports `http` and adds `status_code()` to enums with statuses. |
//! | `axum` | Enables `http` and `serde`, re-exports `axum`, and implements `IntoResponse` for enums with statuses. |
//!
//! ```toml
//! [dependencies]
//! uhoh = { version = "0.1", features = ["axum"] }
//! ```
//!
//! # Formatting messages
//!
//! Tuple fields use positional placeholders such as `{0}`; named fields use
//! placeholders such as `{name}`. Formatting specifiers such as `{0:?}` work
//! as in `format!`. When a tuple message contains placeholders, all its fields
//! are passed as positional arguments and must be used by the format string.
//! A literal message without braces can leave the entire payload unformatted.
//! Named-field messages can use only the fields they need.
//!
//! The displayed message also becomes the serialized response message. Include
//! only context you intend to expose to callers; a fixed message or preset can
//! keep an internal payload out of the response.
//!
//! # Converting errors
//!
//! Put `#[from]` on the sole field of a tuple variant to generate
//! `From<Source>` for the enum. This supports automatic conversion with `?`:
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
//! `#[from]` does not implement error chaining. The generated
//! [`std::error::Error::source`] uses the default implementation and returns
//! `None`, including for transparent variants.
//!
//! # Transparent wrappers
//!
//! `#[error(transparent)]` requires a one-field tuple variant and accepts no
//! message, code, or status overrides. It forwards the wrapped value's display
//! message and `code()`, plus `service_error()` with `serde` and `status_code()`
//! with `http`. The wrapped type must provide those methods for the enabled
//! features; implementing `std::error::Error` alone is insufficient.
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
//! With `http`, transparent variants count as having a status, so every other
//! variant in the enum must also supply one.
//!
//! # HTTP statuses and presets
//!
//! Set `status = NOT_FOUND` to use a `http::StatusCode` constant without an
//! import. Qualified paths and expressions also work, such as
//! `status = uhoh::http::StatusCode::NOT_FOUND`. A bare identifier always names
//! a `StatusCode` constant; qualify application constants, for example
//! `status = crate::MY_STATUS`.
//!
//! Every variant must supply a status, or none may do so. Presets supply both
//! a fixed message and a status; they still require an explicit code:
//!
//! | Preset | Message | Status |
//! | --- | --- | --- |
//! | `ise` | Something went wrong while processing your request. | 500 |
//! | `not_found` | The requested resource was not found. | 404 |
//! | `unauthorized` | You are not authorized to perform that action. | 401 |
//!
//! An explicit `status = ...` overrides a preset's status. Status attributes
//! and presets are accepted without `http`, but do not generate `status_code()`
//! or HTTP responses in that configuration.
//!
//! With `axum`, enums with statuses implement `IntoResponse` using the selected
//! status and a JSON service-error body:
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
//! With `serde`, `service_error()` returns a `ServiceError` containing the
//! code, formatted message, and `details: None`. Transparent variants forward
//! the wrapped value's entire service error, including its details.
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
/// Axum integration, available with the `axum` feature.
pub use axum;
#[cfg(feature = "http")]
/// HTTP types, available with the `http` feature.
pub use http;
pub use uhoh_macros::Error;

/// A serializable application-error body, available with the `serde` feature.
///
/// Derived `service_error()` methods populate `code` and `message` and leave
/// `details` as `None`, serialized as JSON `null`. You can set `details` on the
/// returned value before serializing it. With `axum`, the generated
/// `IntoResponse` implementation creates its own body through `service_error()`.
///
/// This type also implements Serde's `Deserialize`, but its borrowed
/// `&'static str` code requires input that can supply a static string. It is
/// not an owned error type for deserializing arbitrary response buffers.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceError {
    /// The application-defined error code, independent of the display message.
    pub code: &'static str,
    /// The user-facing message produced by the error's `Display` implementation.
    pub message: String,
    /// Optional structured context; `None` serializes as `null`, not an omitted field.
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
