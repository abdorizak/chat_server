use serde::{Deserialize, Serialize};
use validator::Validate;

/// Update user profile input
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileInput {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

/// User search result
#[derive(Debug, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
}
