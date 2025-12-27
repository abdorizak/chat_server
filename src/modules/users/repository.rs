use crate::db::DbPool;
use crate::modules::auth::model::User;
use crate::modules::users::model::UserSearchResult;
use tokio_postgres::types::ToSql;

pub struct UserRepository;

impl UserRepository {
    /// Update user profile
    pub async fn update_profile(
        pool: &DbPool,
        user_id: i32,
        first_name: Option<String>,
        last_name: Option<String>,
        phone: Option<String>,
    ) -> Result<User, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        // Build dynamic update query
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn ToSql + Sync>> = Vec::new();
        
        // Add user_id as first param
        params.push(Box::new(user_id));
        let mut param_index = 2;

        if let Some(fn_val) = first_name {
            updates.push(format!("first_name = ${}", param_index));
            params.push(Box::new(fn_val));
            param_index += 1;
        }

        if let Some(ln_val) = last_name {
            updates.push(format!("last_name = ${}", param_index));
            params.push(Box::new(ln_val));
            param_index += 1;
        }

        if let Some(ph_val) = phone {
            updates.push(format!("phone = ${}", param_index));
            params.push(Box::new(ph_val));
        }

        if updates.is_empty() {
            return Err("No fields to update".into());
        }

        updates.push("updated_at = CURRENT_TIMESTAMP".to_string());

        let query = format!(
            "UPDATE users SET {} WHERE id = $1 
             RETURNING id, username, email, first_name, last_name, phone, password_hash, 
                      created_at, updated_at, last_seen, is_active",
            updates.join(", ")
        );

        // Convert params to slice of references for query
        let param_refs: Vec<&(dyn ToSql + Sync)> = params.iter().map(|p| p.as_ref()).collect();

        let row = client.query_one(&query, &param_refs).await?;

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

    /// Search users by username or email
    pub async fn search_users(
        pool: &DbPool,
        query: &str,
        limit: i64,
    ) -> Result<Vec<UserSearchResult>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let search_pattern = format!("%{}%", query);

        let rows = client
            .query(
                "SELECT id, username, email, first_name, last_name, is_active 
                 FROM users 
                 WHERE (username ILIKE $1 OR email ILIKE $1) AND is_active = true
                 ORDER BY username
                 LIMIT $2",
                &[&search_pattern, &limit],
            )
            .await?;

        let results = rows
            .iter()
            .map(|row| UserSearchResult {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2),
                first_name: row.get(3),
                last_name: row.get(4),
                is_active: row.get(5),
            })
            .collect();

        Ok(results)
    }
}
