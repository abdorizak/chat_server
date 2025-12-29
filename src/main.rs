mod db;
mod logger;
mod common;
mod utils;
mod modules;

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    logger::init();

    log::info!("Starting Chat Server...");

    // Get server configuration
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid number");

    // Create database pool
    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    // Run migrations
    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    log::info!("Database migrations completed");

    // Create pool data
    let pool_data = web::Data::new(pool);

    // Initialize Chat Server (Hub)
    let chat_server = modules::ws::ChatServer::new();
    let chat_server_data = web::Data::new(chat_server);

    log::info!("Server starting at http://{}:{}", host, port);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(pool_data.clone())
            .app_data(chat_server_data.clone())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .wrap(modules::auth::AuthMiddleware)
            .service(
                web::scope("/api")
                    .configure(modules::configure_auth)
                    .configure(modules::configure_users)
                    .configure(modules::configure_contacts)
                    .configure(modules::configure_chats)
            )
            .configure(modules::configure_ws)
            .route("/health", web::get().to(|| async { "OK" }))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
