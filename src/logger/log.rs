use log::{info, warn, error};

/// Initialize logger
pub fn init() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("Logger initialized");
}

/// Log information message
#[allow(dead_code)]
pub fn log_info(message: &str) {
    info!("{}", message);
}

/// Log warning message
#[allow(dead_code)]
pub fn log_warning(message: &str) {
    warn!("{}", message);
}

/// Log error message
#[allow(dead_code)]
pub fn log_error(message: &str) {
    error!("{}", message);
}
