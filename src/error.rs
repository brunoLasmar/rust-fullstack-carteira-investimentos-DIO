use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing Authorization Headers")]
    MissingAuthorization,
    #[error("Invalid Credentials")]
    InvalidCredentials,
    #[error("Asset does not exist")]
    AssetDoesNotExist,
    #[error("User does not exist")]
    UserDoesNotExist,
    #[error("This username is already registered")]
    UsernameTaken,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Template(#[from] askama::Error),
    #[error(transparent)]
    Jwt(#[from] jwt_simple::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::UsernameTaken | Self::MissingAuthorization => StatusCode::BAD_REQUEST,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AssetDoesNotExist | Self::UserDoesNotExist => StatusCode::NOT_FOUND,
            Self::Database(error) => {
                tracing::error!(error = ?error, "database error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Template(error) => {
                tracing::error!(error = ?error, "template rendering error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Jwt(error) => {
                tracing::error!(error = ?error, "JWT error");
                StatusCode::UNAUTHORIZED
            }
        };

        let error_response = ErrorResponse {
            error: self.public_message().to_owned(),
        };

        (status, Json(error_response)).into_response()
    }
}

impl AppError {
    pub fn public_message(&self) -> &'static str {
        match self {
            Self::MissingAuthorization => "Missing Authorization Headers",
            Self::InvalidCredentials => "Invalid Credentials",
            Self::AssetDoesNotExist => "Asset does not exist",
            Self::UserDoesNotExist => "User does not exist",
            Self::UsernameTaken => "This username is already registered",
            Self::Database(_) => "an internal database error occurred",
            Self::Template(_) => "an internal rendering error occurred",
            Self::Jwt(_) => "an internal authentication error occurred",
        }
    }
}
