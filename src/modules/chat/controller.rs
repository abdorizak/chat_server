use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use crate::db::DbPool;
use crate::common::{ApiResponse, ErrorResponse};
use crate::modules::chat::repository::MessageRepository;

// Helper to extract user_id (same hack as contacts module, in real app usage middleware)
fn extract_user_id(req: &HttpRequest) -> Option<i32> {
    req.extensions().get::<i32>().copied().or_else(|| {
        req.headers().get("X-User-Id")
           .and_then(|h| h.to_str().ok())
           .and_then(|s| s.parse::<i32>().ok())
           .filter(|&id| id > 0)
    })
}

/// GET /api/chats/{partner_id}/messages?limit=20&offset=0
#[derive(serde::Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_chat_history(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    query: web::Query<HistoryQuery>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req) {
        Some(id) => id,
        None => return ErrorResponse::unauthorized("Unauthorized"),
    };
    
    let partner_id = path.into_inner();
    let limit = query.limit.unwrap_or(50).min(100); // Max 100
    let offset = query.offset.unwrap_or(0);

    match MessageRepository::get_messages(&pool, user_id, partner_id, limit, offset).await {
        Ok(messages) => ApiResponse::success("Messages retrieved", messages),
        Err(e) => {
            log::error!("Get messages error: {}", e);
            ErrorResponse::internal_error("Failed to retrieve messages")
        }
    }
}

/// POST /api/chats/groups
pub async fn create_group(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    input: web::Json<crate::modules::chat::model::CreateGroupInput>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req) {
        Some(id) => id,
        None => return ErrorResponse::unauthorized("Unauthorized"),
    };

    match MessageRepository::create_group(
        &pool, 
        user_id, 
        &input.name, 
        input.description.clone(), 
        input.members.clone()
    ).await {
        Ok(group) => ApiResponse::success("Group created", group),
        Err(e) => ErrorResponse::internal_error(&format!("Failed to create group: {}", e)),
    }
}

/// GET /api/chats/groups
pub async fn get_groups(
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match extract_user_id(&req) {
        Some(id) => id,
        None => return ErrorResponse::unauthorized("Unauthorized"),
    };

    match MessageRepository::get_user_groups(&pool, user_id).await {
        Ok(groups) => ApiResponse::success("Groups retrieved", groups),
        Err(e) => ErrorResponse::internal_error(&e.to_string()),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/chats")
            .route("/{partner_id}/messages", web::get().to(get_chat_history))
            .route("/groups", web::post().to(create_group))
            .route("/groups", web::get().to(get_groups))
    );
}
