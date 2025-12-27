use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use actix_ws::Session;

/// Shared chat server state to manage active connections
#[derive(Clone)]
pub struct ChatServer {
    /// Map of User ID -> WebSocket Session
    sessions: Arc<RwLock<HashMap<i32, Session>>>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new session for a user
    pub fn join(&self, user_id: i32, session: Session) {
        log::info!("User {} joined chat", user_id);
        self.sessions.write().unwrap().insert(user_id, session);
    }

    /// Remove a user session
    pub fn leave(&self, user_id: i32) {
        log::info!("User {} left chat", user_id);
        self.sessions.write().unwrap().remove(&user_id);
    }

    /// Send a message to a specific user if they are connected
    pub async fn send_message(&self, user_id: i32, message: &str) {
        if let Some(session) = self.sessions.read().unwrap().get(&user_id) {
            let mut session = session.clone();
            let _ = session.text(message).await;
        }
    }

    /// Broadcast message to multiple users
    pub async fn broadcast(&self, user_ids: &[i32], message: &str) {
        let sessions = self.sessions.read().unwrap();
        for user_id in user_ids {
            if let Some(session) = sessions.get(user_id) {
                let mut session = session.clone();
                let message = message.to_string();
                actix_rt::spawn(async move {
                    let _ = session.text(message).await;
                });
            }
        }
    }
}
