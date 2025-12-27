use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use validator::Validate;

use crate::db::DbPool;
use crate::common::{ApiResponse, ErrorResponse};
use crate::modules::auth::model::{RegisterInput, LoginInput};
use crate::modules::auth::services::AuthService;

/// POST /api/auth/register - Register new user
pub async fn register(
    pool: web::Data<DbPool>,
    input: web::Json<RegisterInput>,
) -> HttpResponse {
    // Validate input
    if let Err(errors) = input.validate() {
        return ErrorResponse::bad_request(&format!("Validation error: {:?}", errors));
    }

    match AuthService::register(&pool, input.into_inner()).await {
        Ok(response) => ApiResponse::success("User registered successfully", response),
        Err(e) => {
            log::error!("Registration error: {}", e);
            if e.to_string().contains("already exists") {
                ErrorResponse::bad_request(&e.to_string())
            } else {
                ErrorResponse::internal_error("Failed to register user")
            }
        }
    }
}

/// POST /api/auth/login - Login user
pub async fn login(
    pool: web::Data<DbPool>,
    input: web::Json<LoginInput>,
) -> HttpResponse {
    // Validate input
    if let Err(errors) = input.validate() {
        return ErrorResponse::bad_request(&format!("Validation error: {:?}", errors));
    }

    match AuthService::login(&pool, input.into_inner()).await {
        Ok(response) => ApiResponse::success("Login successful", response),
        Err(e) => {
            log::error!("Login error: {}", e);
            if e.to_string().contains("Invalid") {
                ErrorResponse::unauthorized(&e.to_string())
            } else {
                ErrorResponse::internal_error("Failed to login")
            }
        }
    }
}

/// GET /api/auth/me - Get current user profile
pub async fn get_current_user(
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    // Extract user_id from request extensions (set by middleware)
    match req.extensions().get::<i32>() {
        Some(user_id) => {
            match AuthService::get_user_by_id(&pool, *user_id).await {
                Ok(Some(user)) => {
                    let user_public = crate::modules::auth::model::UserPublic::from(user);
                    ApiResponse::success("User found", user_public)
                }
                Ok(None) => ErrorResponse::not_found("User not found"),
                Err(e) => {
                    log::error!("Error fetching user: {}", e);
                    ErrorResponse::internal_error("Failed to fetch user")
                }
            }
        }
        None => ErrorResponse::unauthorized("Unauthorized"),
    }
}

/// Configure auth routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/me", web::get().to(get_current_user)),
    );
}
