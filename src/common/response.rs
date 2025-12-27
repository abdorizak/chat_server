use actix_web::{HttpResponse, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(message: &str, data: T) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: message.to_string(),
            data: Some(data),
        })
    }

    #[allow(dead_code)]
    pub fn success_no_data(message: &str) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse::<()> {
            success: true,
            message: message.to_string(),
            data: None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
    pub error: String,
}

impl ErrorResponse {
    pub fn bad_request(message: &str) -> HttpResponse {
        HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            message: message.to_string(),
            error: "Bad Request".to_string(),
        })
    }

    pub fn unauthorized(message: &str) -> HttpResponse {
        HttpResponse::Unauthorized().json(ErrorResponse {
            success: false,
            message: message.to_string(),
            error: "Unauthorized".to_string(),
        })
    }

    pub fn not_found(message: &str) -> HttpResponse {
        HttpResponse::NotFound().json(ErrorResponse {
            success: false,
            message: message.to_string(),
            error: "Not Found".to_string(),
        })
    }

    pub fn internal_error(message: &str) -> HttpResponse {
        HttpResponse::InternalServerError().json(ErrorResponse {
            success: false,
            message: message.to_string(),
            error: "Internal Server Error".to_string(),
        })
    }

    #[allow(dead_code)]
    pub fn custom(status: StatusCode, message: &str, error: &str) -> HttpResponse {
        HttpResponse::build(status).json(ErrorResponse {
            success: false,
            message: message.to_string(),
            error: error.to_string(),
        })
    }
}
