use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use validator::Validate;

use crate::db::DbPool;
use crate::common::{ApiResponse, ErrorResponse};
use crate::modules::users::model::UpdateProfileInput;
use crate::modules::users::repository::UserRepository;

/// PUT /api/users/me - Update user profile
pub async fn update_profile(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    input: web::Json<UpdateProfileInput>,
) -> HttpResponse {
    // Extract user_id from request extensions (set by auth middleware - TODO)
    // For now, we'll assume it's passed or extract from dummy implementation
    // In a real app, middleware decodes JWT and sets this
    let user_id = match req.extensions().get::<i32>() {
        Some(id) => *id,
        None => return ErrorResponse::unauthorized("Unauthorized"),
    };

    if let Err(errors) = input.validate() {
        return ErrorResponse::bad_request(&format!("Validation error: {:?}", errors));
    }

    match UserRepository::update_profile(
        &pool, 
        user_id, 
        input.first_name.clone(),
        input.last_name.clone(), 
        input.phone.clone()
    ).await {
        Ok(user) => {
             let user_public = crate::modules::auth::model::UserPublic::from(user);
             ApiResponse::success("Profile updated successfully", user_public)
        },
        Err(e) => {
            log::error!("Update profile error: {}", e);
            ErrorResponse::internal_error("Failed to update profile")
        }
    }
}

/// GET /api/users/search?q=query - Search users
#[derive(serde::Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub async fn search_users(
    pool: web::Data<DbPool>,
    query: web::Query<SearchQuery>,
) -> HttpResponse {
    if query.q.len() < 3 {
        return ErrorResponse::bad_request("Search query must be at least 3 characters");
    }

    match UserRepository::search_users(&pool, &query.q, 20).await {
        Ok(users) => ApiResponse::success("Users found", users),
        Err(e) => {
            log::error!("User search error: {}", e);
            ErrorResponse::internal_error("Failed to search users")
        }
    }
}

/// Configure users routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/me", web::put().to(update_profile))
            .route("/search", web::get().to(search_users)),
    );
}
