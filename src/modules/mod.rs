pub mod auth;
pub mod users;
pub mod ws;
pub mod chat;
pub mod contacts;

// Export module configurations
pub use auth::configure as configure_auth;
pub use users::configure as configure_users;
pub use ws::configure as configure_ws;
pub use contacts::configure as configure_contacts;
pub use chat::configure as configure_chats;
