use uhoh::Error;

#[derive(Debug, Error)]
enum WorkerError {
    #[error("Job {0} failed.", code = "W201")]
    Failed(String),
}

#[test]
fn basic_errors_work_without_optional_features() {
    let error = WorkerError::Failed("daily-report".into());
    assert_eq!(error.to_string(), "Job daily-report failed.");
    assert_eq!(error.code(), "W201");
    let _: &dyn std::error::Error = &error;
}

#[cfg(feature = "serde")]
#[test]
fn service_errors_serialize_without_axum() {
    let error = WorkerError::Failed("daily-report".into());
    assert_eq!(
        serde_json::to_value(error.service_error()).unwrap(),
        serde_json::json!({
            "code": "W201",
            "message": "Job daily-report failed.",
            "details": null,
        }),
    );
}

#[cfg(feature = "http")]
#[derive(Debug, Error)]
enum HttpError {
    #[error(not_found, code = "H404")]
    Missing,
}

#[cfg(feature = "http")]
#[test]
fn status_codes_work_without_axum() {
    assert_eq!(
        HttpError::Missing.status_code(),
        uhoh::http::StatusCode::NOT_FOUND,
    );
}

#[cfg(not(feature = "http"))]
#[derive(Debug, Error)]
enum WrappedError {
    #[error(transparent)]
    Worker(#[from] WorkerError),
}

#[cfg(not(feature = "http"))]
#[test]
fn transparent_errors_work_without_http() {
    let error: WrappedError = WorkerError::Failed("daily-report".into()).into();
    assert_eq!(error.code(), "W201");
    assert_eq!(error.to_string(), "Job daily-report failed.");
    #[cfg(feature = "serde")]
    assert_eq!(error.service_error().code, "W201");
}
