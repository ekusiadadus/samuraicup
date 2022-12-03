use crate::error::*;

#[derive(Debug)]
pub enum RepositoryError {
    RecordNotFound,
    InvalidRecord,
    SerializationError,
}

impl IServiceError for RepositoryError {
    fn error_type(&self) -> String {
        use RepositoryError::*;

        match self {
            RecordNotFound => "record_not_found",
            InvalidRecord => "invalid_record",
            SerializationError => "serialization_error",
        }
        .to_string()
    }

    fn status_code(&self) -> http::StatusCode {
        use RepositoryError::*;

        match self {
            RecordNotFound => http::StatusCode::NOT_FOUND,
            InvalidRecord => http::StatusCode::INTERNAL_SERVER_ERROR,
            SerializationError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
