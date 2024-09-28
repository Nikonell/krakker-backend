use std::fmt::Display;
use actix_web::{http::StatusCode, ResponseError};
use apistos::{ApiComponent, ApiErrorComponent};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ApiErrorComponent)]
#[openapi_error(
    status(code = 400),
    status(code = 405),
    status(code = 404),
    status(code = 401),
    status(code = 500),
)]
pub enum ErrorResponse {
    BadRequest(String),
    MethodNotAllowed(String),
    NotFound(String),
    Unauthorized(String),
    InternalServerError(String),
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ErrorResponse::BadRequest(message) => message,
            ErrorResponse::MethodNotAllowed(message) => message,
            ErrorResponse::NotFound(message) => message,
            ErrorResponse::Unauthorized(message) => message,
            ErrorResponse::InternalServerError(message) => message,
        };
        write!(f, r#"{{"status": "error", "message": "{}"}}"#, message)
    }
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ErrorResponse::BadRequest(_) => StatusCode::BAD_REQUEST,
            ErrorResponse::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            ErrorResponse::NotFound(_) => StatusCode::NOT_FOUND,
            ErrorResponse::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ErrorResponse::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, ApiComponent)]
pub struct SuccessResponse<T: JsonSchema> {
    pub status: &'static str,
    pub data: T,
}

impl<T: JsonSchema> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            status: "success",
            data
        }
    }
}
