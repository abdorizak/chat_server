pub mod model;
pub mod repository;
pub mod services;
pub mod middleware;

pub use middleware::AuthMiddleware;
pub mod controller;

pub use controller::configure;
