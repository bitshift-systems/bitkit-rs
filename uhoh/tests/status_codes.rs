#![cfg(feature = "http")]

use uhoh::Error;

#[derive(Debug, Error)]
enum DatabaseError {
    #[error("Query failure: {0}", code = "D0002", status = INTERNAL_SERVER_ERROR)]
    Query(String),

    #[error("Missing record", code = "D0404", status = NOT_FOUND)]
    Missing,

    #[error(ise, code = "D0400", status = BAD_REQUEST)]
    Invalid,

    #[error("Conflict", code = "D0409", status = uhoh::http::StatusCode::CONFLICT)]
    Conflict,

    #[error("Custom status", code = "D0429", status = uhoh::http::StatusCode::from_u16(429).unwrap())]
    Custom,
}

#[test]
fn status_names_expand_without_imports_and_preserve_expressions() {
    assert_eq!(
        DatabaseError::Query("bad query".into())
            .status_code()
            .as_u16(),
        500
    );
    assert_eq!(DatabaseError::Missing.status_code().as_u16(), 404);
    assert_eq!(DatabaseError::Invalid.status_code().as_u16(), 400);
    assert_eq!(DatabaseError::Conflict.status_code().as_u16(), 409);
    assert_eq!(DatabaseError::Custom.status_code().as_u16(), 429);
}

#[cfg(feature = "axum")]
#[test]
fn status_names_set_the_response_status() {
    use uhoh::axum::response::IntoResponse;

    assert_eq!(
        DatabaseError::Missing.into_response().status().as_u16(),
        404
    );
}
