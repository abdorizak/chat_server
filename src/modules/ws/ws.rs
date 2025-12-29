use actix_ws::Message;
use futures_util::{
    future::{select, Either},
    StreamExt as _,
};
use tokio::{pin, time::interval};
use std::time::{Duration, Instant};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use crate::modules::ws::type_def::WsMessage;
use crate::modules::ws::server::ChatServer;
use crate::db::DbPool;
use crate::modules::chat::MessageRepository;

/// WebSocket handshake and start endpoint
pub async fn start_connection(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<ChatServer>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let (res, session, stream) = actix_ws::handle(&req, stream)?;

    // Ideally, extract user_id from JWT here
    // For now, we'll parse it from query params: /ws?userId=1
    // Fallback to 0 if not provided (should be handled better in prod)
    let q_str = req.query_string();
    let user_id = qvec::extract_user_id_from_query(q_str).unwrap_or(0);
    
    if user_id == 0 {
        log::warn!("Connection rejected: No user_id provided");
        return Ok(HttpResponse::Unauthorized().finish());
    }

    // Register session
    srv.join(user_id, session.clone());

    // Spawn websocket handler task
    actix_rt::spawn(async move {
        let mut tick_interval = interval(Duration::from_secs(5));
        let mut last_heartbeat = Instant::now();
        let mut session = session.clone();
        
        pin!(stream);

        loop {
            let tick = tick_interval.tick();
            pin!(tick);

            // Wait for either stream message or heartbeat tick
            match select(stream.next(), tick).await {
                Either::Left((Some(Ok(msg)), _)) => {
                    match msg {
                        Message::Text(text) => {
                            // Parse incoming message
                            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                                match ws_msg {
                                    WsMessage::TextMessage { to_user_id, content } => {
                                        log::info!("Message from {} to {}: {}", user_id, to_user_id, content);
                                        
                                        // 1. Save to Database
                                        let save_result = MessageRepository::create_message(
                                            &pool, 
                                            user_id, 
                                            to_user_id, 
                                            &content
                                        ).await;

                                        match save_result {
                                            Ok(_saved_msg) => {
                                                // 2. Route to Recipient (if online)
                                                let payload = serde_json::to_string(&WsMessage::TextMessage {
                                                    to_user_id: user_id, // From sender perspective
                                                    content: content.clone(),
                                                }).unwrap_or_default();
                                                
                                                srv.send_message(to_user_id, &payload).await;
                                                
                                                // 3. Ack to Sender
                                                let _ = session.text(format!("Sent: {}", content)).await;
                                            },
                                            Err(e) => {
                                                log::error!("Failed to save message: {}", e);
                                                let _ = session.text("Error: Failed to send message").await;
                                            }
                                        }
                                    },
                                    WsMessage::GroupMessage { group_id, content } => {
                                        log::info!("Group Message from {} to group {}: {}", user_id, group_id, content);

                                        // 1. Save to Group DB
                                        let save_result = MessageRepository::create_group_message(
                                            &pool,
                                            user_id,
                                            group_id,
                                            &content
                                        ).await;

                                        match save_result {
                                            Ok(_saved_msg) => {
                                                // 2. Get Members
                                                if let Ok(members) = MessageRepository::get_group_members(&pool, group_id).await {
                                                    // 3. Broadcast to all members (except sender ideally, but broadcast handles connection check)
                                                    let payload = serde_json::to_string(&WsMessage::GroupMessage {
                                                        group_id,
                                                        content: format!("{}: {}", user_id, content), // Simple format for now
                                                    }).unwrap_or_default();
                                                    
                                                    // Filter out sender from broadcast list to avoid duplicate echo
                                                    let recipients: Vec<i32> = members.into_iter().filter(|&id| id != user_id).collect();
                                                    srv.broadcast(&recipients, &payload).await;
                                                    
                                                    let _ = session.text(format!("Sent Group: {}", content)).await;
                                                }
                                            },
                                            Err(e) => {
                                                 log::error!("Failed to save group message: {}", e);
                                                 let _ = session.text(format!("Error: {}", e)).await;
                                            }
                                        }
                                    }
                                    WsMessage::Typing { conversation_id, group_id, is_typing } => {
                                        // 1. Group Typing
                                        if let Some(g_id) = group_id {
                                            if let Ok(members) = MessageRepository::get_group_members(&pool, g_id).await {
                                                let payload = serde_json::to_string(&WsMessage::Typing {
                                                    conversation_id: None,
                                                    group_id: Some(g_id),
                                                    is_typing,
                                                }).unwrap_or_default();
                                                
                                                let recipients: Vec<i32> = members.into_iter().filter(|&id| id != user_id).collect();
                                                srv.broadcast(&recipients, &payload).await;
                                            }
                                        }
                                        // 2. 1-to-1 Typing
                                        else if let Some(c_id) = conversation_id {
                                             if let Ok(Some(partner_id)) = MessageRepository::get_conversation_partner(&pool, c_id, user_id).await {
                                                 let payload = serde_json::to_string(&WsMessage::Typing {
                                                    conversation_id: Some(c_id),
                                                    group_id: None,
                                                    is_typing,
                                                }).unwrap_or_default();
                                                
                                                srv.send_message(partner_id, &payload).await;
                                             }
                                        }
                                    },
                                    WsMessage::MessageRead { message_id } => {
                                        // 1. Mark in DB
                                        if let Ok(Some(sender_id)) = MessageRepository::mark_message_read(&pool, message_id, user_id).await {
                                            // 2. Notify Sender
                                            let payload = serde_json::to_string(&WsMessage::MessageRead {
                                                message_id
                                            }).unwrap_or_default();
                                            
                                            srv.send_message(sender_id, &payload).await;
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                        Message::Ping(bytes) => {
                            last_heartbeat = Instant::now();
                            let _ = session.pong(&bytes).await;
                        }
                        Message::Pong(_) => {
                            last_heartbeat = Instant::now();
                        }
                        Message::Close(reason) => {
                            srv.leave(user_id);
                            let _ = session.close(reason).await;
                            break;
                        }
                        _ => {}
                    }
                }
                Either::Left((Some(Err(e)), _)) => {
                    log::error!("WS error: {}", e);
                    srv.leave(user_id);
                    break;
                }
                Either::Left((None, _)) => {
                    srv.leave(user_id);
                    break;
                },
                Either::Right((_inst, _)) => {
                    // Check heartbeat
                    if last_heartbeat.elapsed() > Duration::from_secs(10) {
                         log::info!("WS client heartbeat timed out");
                         srv.leave(user_id);
                         let _ = session.close(None).await;
                         break;
                    }
                    let _ = session.ping(b"").await;
                }
            }
        }
    });

    Ok(res)
}

// Helper to handle query extraction manually since we're inside the handler
mod qvec {
    pub fn extract_user_id_from_query(query: &str) -> Option<i32> {
        let params: Vec<(String, String)> = serde_urlencoded::from_str(query).ok()?;
        for (k, v) in params {
            if k == "userId" {
                return v.parse::<i32>().ok();
            }
        }
        None
    }
}

/// Configure WS routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/ws").route(web::get().to(start_connection))
    );
}
