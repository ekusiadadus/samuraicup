pub use crate::repository::RepositoryError;
use anyhow::Error;
use serde::*;
use std::any::Any;

pub trait IServiceError: Any {
    fn error_type(&self) -> String {
        "internal_server_error".to_string()
    }

    fn status_code(&self) -> http::StatusCode {
        http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug)]
pub struct ServiceError {
    type_id: std::any::TypeId,
    error_type: String,
    status_code: http::StatusCode,
    inner: Error,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    error_type: String,
    error: String,
}

impl ServiceError {
    pub fn new<E>(err: impl IServiceError, detail: E) -> ServiceError
    where
        Error: From<E>,
    {
        ServiceError {
            type_id: err.type_id(),
            error_type: err.error_type(),
            status_code: err.status_code(),
            inner: From::from(detail),
        }
    }

    pub fn only(err: impl IServiceError) -> ServiceError {
        ServiceError {
            type_id: err.type_id(),
            error_type: err.error_type(),
            status_code: err.status_code(),
            inner: Error::msg("error"),
        }
    }

    pub fn into_inner(self) -> Error {
        self.inner
    }

    pub fn status_code(&self) -> http::StatusCode {
        self.status_code
    }

    pub fn error_type(&self) -> String {
        self.error_type.clone()
    }

    pub fn is_error_of(&self, err: impl IServiceError) -> bool {
        self.type_id == err.type_id() && self.error_type() == err.error_type()
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            error_type: self.error_type.clone(),
            error: format!("{:#?}", self.inner),
        }
    }

    // 500のときは詳細なエラーは隠す
    pub fn to_secure_error_response(&self) -> ErrorResponse {
        if self.status_code == http::StatusCode::INTERNAL_SERVER_ERROR {
            ErrorResponse {
                error_type: self.error_type.clone(),
                error: String::new(),
            }
        } else {
            self.to_error_response()
        }
    }

    pub fn to_http_response(self) -> hyper::Response<hyper::Body> {
        hyper::Response::builder()
            .status(self.status_code)
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(hyper::Body::from(
                serde_json::to_string(&self.to_secure_error_response()).unwrap(),
            ))
            .unwrap()
    }
}

// failure::Error can be treated as ServiceError
impl IServiceError for Error {}

pub enum FutureError {
    JoinError,
}

// for tokio::task::spawn_blocking
impl IServiceError for FutureError {
    fn error_type(&self) -> String {
        match self {
            FutureError::JoinError => "internal_server_error".to_string(),
        }
    }

    fn status_code(&self) -> http::StatusCode {
        match self {
            FutureError::JoinError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<tokio::task::JoinError> for ServiceError {
    fn from(err: tokio::task::JoinError) -> ServiceError {
        ServiceError::new(FutureError::JoinError, err)
    }
}

// アプリ全体でよく使いそうなエラーはここで定義しても良い
// 注意: ビジネスロジックを入れないこと。また、エラーハンドリングをしたくなるような時はサボらずカスタムエラーを定義すること
pub enum GeneralError {
    SerializationError,
    InvalidAuthority,
}

impl GeneralError {
    pub fn serialization_error<E>(detail: E) -> ServiceError
    where
        Error: From<E>,
    {
        ServiceError::new(GeneralError::SerializationError, detail)
    }

    pub fn invalid_authority<E>(detail: E) -> ServiceError
    where
        Error: From<E>,
    {
        ServiceError::new(GeneralError::InvalidAuthority, detail)
    }
}

impl IServiceError for GeneralError {
    fn error_type(&self) -> String {
        use GeneralError::*;

        match self {
            SerializationError => "serialization_error".to_string(),
            InvalidAuthority => "invalid_authority".to_string(),
        }
    }

    fn status_code(&self) -> http::StatusCode {
        use GeneralError::*;

        match self {
            SerializationError => http::StatusCode::BAD_REQUEST,
            InvalidAuthority => http::StatusCode::UNAUTHORIZED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum E {
        E1,
        E2,
    }

    impl IServiceError for E {
        fn error_type(&self) -> String {
            use E::*;

            match self {
                E1 => "e1",
                E2 => "e2",
            }
            .to_string()
        }

        fn status_code(&self) -> http::StatusCode {
            use E::*;

            match self {
                E1 => http::StatusCode::INTERNAL_SERVER_ERROR,
                E2 => http::StatusCode::BAD_REQUEST,
            }
        }
    }

    #[test]
    fn it_should_handle_errors() {
        let err = ServiceError::only(E::E1);
        assert_eq!(err.error_type(), "e1".to_string());
        assert!(err.is_error_of(E::E1));
        assert!(!err.is_error_of(E::E2));
    }

    #[derive(PartialEq, Debug)]
    enum F {
        E1,
    }

    impl IServiceError for F {
        fn error_type(&self) -> String {
            use F::*;

            match self {
                E1 => "e1",
            }
            .to_string()
        }

        fn status_code(&self) -> http::StatusCode {
            use F::*;

            match self {
                E1 => http::StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    }

    #[test]
    fn it_should_distinguish_between_different_types_with_same_name() {
        let e1 = ServiceError::only(E::E1);
        let e2 = ServiceError::only(F::E1);

        assert!(!e1.is_error_of(F::E1));
        assert!(!e2.is_error_of(E::E1));
    }
}
