use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub user_id: i32,
    pub contact_id: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Included user details for frontend convenience
    pub contact_user: Option<ContactUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactUser {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct ContactRequestInput {
    pub username: String, // identify user by username
}
