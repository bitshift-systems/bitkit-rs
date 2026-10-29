#![cfg(feature = "http")]

use uhoh::{Error, http::StatusCode};

#[derive(Debug, Error)]
enum ExternalPresetError {
    #[error(ise, code = "R0005")]
    Unknown,

    #[error(not_found, code = "R0404")]
    NotFound,

    #[error(unauthorized, code = "R0401")]
    Unauthorized,
}

#[derive(Debug, Error)]
enum RepositoryError {
    #[error(not_found, code = "R0404")]
    NotFound,
}

#[derive(Debug, Error)]
enum RequestError {
    #[error(transparent)]
    Repo(#[from] RepositoryError),
}

#[test]
fn presets_expand_in_downstream_crates() {
    assert_eq!(
        ExternalPresetError::Unknown.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        ExternalPresetError::NotFound.status_code(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        ExternalPresetError::Unauthorized.status_code(),
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn transparent_variants_preserve_the_wrapped_error() {
    let error: RequestError = RepositoryError::NotFound.into();

    assert_eq!(error.to_string(), "The requested resource was not found.");
    assert_eq!(error.code(), "R0404");
    assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    #[cfg(feature = "serde")]
    assert_eq!(error.service_error().code, "R0404");
}
