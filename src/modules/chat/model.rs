use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: i32,
    pub conversation_id: i32,
    pub sender_id: i32,
    pub content: String,
    pub message_type: String,
    pub sent_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CreateMessageInput {
    pub recipient_id: i32,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub creator_id: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupInput {
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<i32>, // Initial members to add
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct GroupMember {
    pub group_id: i32,
    pub user_id: i32,
    pub role: String, // admin, member
    pub joined_at: DateTime<Utc>,
}
