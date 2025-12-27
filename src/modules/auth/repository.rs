use crate::db::DbPool;
use crate::modules::auth::model::{User, AuthToken};
use chrono::{DateTime, Utc};

pub struct AuthRepository;

impl AuthRepository {
    /// Create a new user
    pub async fn create_user(
        pool: &DbPool,
        username: &str,
        email: &str,
        first_name: &str,
        last_name: &str,
        phone: Option<&str>,
        password_hash: &str,
    ) -> Result<User, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO users (username, email, first_name, last_name, phone, password_hash) 
                 VALUES ($1, $2, $3, $4, $5, $6) 
                 RETURNING id, username, email, first_name, last_name, phone, password_hash, 
                          created_at, updated_at, last_seen, is_active",
                &[&username, &email, &first_name, &last_name, &phone, &password_hash],
            )
            .await?;

        Ok(User {
            id: row.get(0),
            username: row.get(1),
            email: row.get(2),
            first_name: row.get(3),
            last_name: row.get(4),
            phone: row.get(5),
            password_hash: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
            last_seen: row.get(9),
            is_active: row.get(10),
        })
    }

    /// Find user by email
    pub async fn find_by_email(
        pool: &DbPool,
        email: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let result = client
            .query_opt(
                "SELECT id, username, email, first_name, last_name, phone, password_hash, 
                        created_at, updated_at, last_seen, is_active 
                 FROM users WHERE email = $1",
                &[&email],
            )
            .await?;

        match result {
            Some(row) => Ok(Some(User {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                first_name: row.get(3),
                last_name: row.get(4),
                phone: row.get(5),
                password_hash: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
                last_seen: row.get(9),
                is_active: row.get(10),
            })),
            None => Ok(None),
        }
    }

    /// Find user by ID
    pub async fn find_by_id(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Option<User>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let result = client
            .query_opt(
                "SELECT id, username, email, first_name, last_name, phone, password_hash, 
                        created_at, updated_at, last_seen, is_active 
                 FROM users WHERE id = $1",
                &[&user_id],
            )
            .await?;

        match result {
            Some(row) => Ok(Some(User {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                first_name: row.get(3),
                last_name: row.get(4),
                phone: row.get(5),
                password_hash: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
                last_seen: row.get(9),
                is_active: row.get(10),
            })),
            None => Ok(None),
        }
    }

    /// Store auth token
    pub async fn store_token(
        pool: &DbPool,
        user_id: i32,
        token_hash: &str,
        refresh_token_hash: &str,
        expires_at: DateTime<Utc>,
        device_info: Option<&str>,
    ) -> Result<AuthToken, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO auth_tokens (user_id, token_hash, refresh_token_hash, expires_at, device_info) 
                 VALUES ($1, $2, $3, $4, $5) 
                 RETURNING id, user_id, token_hash, refresh_token_hash, expires_at, created_at, revoked, device_info",
                &[&user_id, &token_hash, &refresh_token_hash, &expires_at, &device_info],
            )
            .await?;

        Ok(AuthToken {
            id: row.get(0),
            user_id: row.get(1),
            token_hash: row.get(2),
            refresh_token_hash: row.get(3),
            expires_at: row.get(4),
            created_at: row.get(5),
            revoked: row.get(6),
            device_info: row.get(7),
        })
    }

    /// Update last seen timestamp
    pub async fn update_last_seen(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE users SET last_seen = CURRENT_TIMESTAMP WHERE id = $1",
                &[&user_id],
            )
            .await?;

        Ok(())
    }
}
