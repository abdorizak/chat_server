use serde::{Deserialize, Serialize};

/// WebSocket message types
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Text message
    TextMessage {
        to_user_id: i32,
        content: String,
    },
    GroupMessage {
        group_id: i32,
        content: String,
    },
    /// Typing indicator
    Typing {
        conversation_id: Option<i32>,
        group_id: Option<i32>,
        is_typing: bool,
    },
    /// Read receipt
    MessageRead {
        message_id: i32,
    },
    /// User status
    UserStatus {
        user_id: i32,
        status: String, // online, offline, away
    },
}

/// WebSocket client connection info
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WsClient {
    pub user_id: i32,
    pub connection_id: String,
}
