use crate::db::DbPool;
use crate::modules::auth::model::{User, RegisterInput, LoginInput, AuthResponse, UserPublic};
use crate::modules::auth::repository::AuthRepository;
use crate::utils::{hash_password, verify_password, generate_jwt, generate_refresh_token};
use chrono::{Duration, Utc};

pub struct AuthService;

impl AuthService {
    /// Register a new user
    pub async fn register(
        pool: &DbPool,
        input: RegisterInput,
    ) -> Result<AuthResponse, Box<dyn std::error::Error>> {
        // Check if user already exists
        if let Some(_) = AuthRepository::find_by_email(pool, &input.email).await? {
            return Err("User with this email already exists".into());
        }

        // Hash password
        let password_hash = hash_password(&input.password)?;

        // Create user
        let user = AuthRepository::create_user(
            pool,
            &input.username,
            &input.email,
            &input.first_name,
            &input.last_name,
            input.phone.as_deref(),
            &password_hash,
        )
        .await?;

        // Generate tokens
        let token = generate_jwt(user.id, &user.email)?;
        let refresh_token = generate_refresh_token();

        // Store tokens
        let token_hash = hash_password(&token)?;
        let refresh_token_hash = hash_password(&refresh_token)?;
        let expires_at = Utc::now() + Duration::seconds(3600);

        AuthRepository::store_token(
            pool,
            user.id,
            &token_hash,
            &refresh_token_hash,
            expires_at,
            None,
        )
        .await?;

        Ok(AuthResponse {
            user: UserPublic::from(user),
            token,
            refresh_token,
        })
    }

    /// Login existing user
    pub async fn login(
        pool: &DbPool,
        input: LoginInput,
    ) -> Result<AuthResponse, Box<dyn std::error::Error>> {
        // Find user by email
        let user = AuthRepository::find_by_email(pool, &input.email)
            .await?
            .ok_or("Invalid email or password")?;

        // Verify password
        if !verify_password(&input.password, &user.password_hash)? {
            return Err("Invalid email or password".into());
        }

        // Check if user is active
        if !user.is_active {
            return Err("Account is deactivated".into());
        }

        // Generate tokens
        let token = generate_jwt(user.id, &user.email)?;
        let refresh_token = generate_refresh_token();

        // Store tokens
        let token_hash = hash_password(&token)?;
        let refresh_token_hash = hash_password(&refresh_token)?;
        let expires_at = Utc::now() + Duration::seconds(3600);

        AuthRepository::store_token(
            pool,
            user.id,
            &token_hash,
            &refresh_token_hash,
            expires_at,
            None,
        )
        .await?;

        // Update last seen
        AuthRepository::update_last_seen(pool, user.id).await?;

        Ok(AuthResponse {
            user: UserPublic::from(user),
            token,
            refresh_token,
        })
    }

    /// Get user by ID
    pub async fn get_user_by_id(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Option<User>, Box<dyn std::error::Error>> {
        AuthRepository::find_by_id(pool, user_id).await
    }
}
