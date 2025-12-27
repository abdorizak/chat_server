use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use std::env;

pub type DbPool = Pool;

/// Initialize database connection pool
pub async fn create_pool() -> Result<DbPool, Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pg_config = database_url.parse::<tokio_postgres::Config>()?;
    
    let mut pool_config = Config::new();
    pool_config.dbname = pg_config.get_dbname().map(String::from);
    pool_config.host = pg_config.get_hosts().first().map(|h| match h {
        tokio_postgres::config::Host::Tcp(s) => s.clone(),
        #[cfg(unix)]
        tokio_postgres::config::Host::Unix(p) => p.to_string_lossy().to_string(),
    });
    pool_config.port = pg_config.get_ports().first().copied();
    pool_config.user = pg_config.get_user().map(String::from);
    pool_config.password = pg_config.get_password().map(|p| String::from_utf8_lossy(p).to_string());
    
    pool_config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = pool_config.create_pool(Some(Runtime::Tokio1), NoTls)?;

    log::info!("Database connection pool created successfully");
    
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    
    log::info!("Running database migrations...");
    
    // Read and execute migration files
    let migrations: Vec<&str> = vec![
        include_str!("../../migrations/01_create_users_table.sql"),
        include_str!("../../migrations/02_create_auth_tokens_table.sql"),
        include_str!("../../migrations/03_create_contacts_table.sql"),
        include_str!("../../migrations/04_create_conversations_table.sql"),
        include_str!("../../migrations/05_create_messages_table.sql"),
        include_str!("../../migrations/06_create_groups_table.sql"),
        include_str!("../../migrations/07_create_group_members_table.sql"),
        include_str!("../../migrations/08_create_group_messages_table.sql"),
    ];

    for (index, migration) in migrations.iter().enumerate() {
        log::info!("Running migration {}/{}", index + 1, migrations.len());
        client.batch_execute(*migration).await?;
    }

    log::info!("All migrations completed successfully");
    
    Ok(())
}
