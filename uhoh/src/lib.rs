//! Typed, HTTP-ready application errors.
//!
//! `#[derive(uhoh::Error)]` turns an enum whose variants have an
//! `#[error(message, code = "...")]` attribute into a standard Rust error.
//! Adding `status = ...` to every variant (and enabling the `axum` feature)
//! also implements Axum's `IntoResponse`. The response body is a
//! `uhoh::ServiceError`, available with the `serde` feature.
//!
//! ```ignore
//! #[derive(Debug, uhoh::Error)]
//! enum DatabaseError {
//!     #[error(
//!         "Query failure: {0}",
//!         code = "D201",
//!         status = INTERNAL_SERVER_ERROR
//!     )]
//!     Query(String),
//! }
//! ```

extern crate self as uhoh;

#[cfg(feature = "axum")]
pub use axum;
#[cfg(feature = "http")]
pub use http;
pub use uhoh_macros::Error;

#[cfg(feature = "serde")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceError {
    pub code: &'static str,
    pub message: String,
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
