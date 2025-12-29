use crate::db::DbPool;
use crate::modules::contacts::model::{Contact, ContactUser};

pub struct ContactRepository;

impl ContactRepository {
    /// Send a friend request (create pending status)
    pub async fn send_request(
        pool: &DbPool,
        user_id: i32,
        contact_username: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        // 1. Find contact user ID
        let user_row = client.query_opt(
            "SELECT id FROM users WHERE username = $1",
            &[&contact_username],
        ).await?;

        let contact_id: i32 = match user_row {
            Some(row) => row.get(0),
            None => return Err("User not found".into()),
        };

        if contact_id == user_id {
            return Err("Cannot add yourself".into());
        }

        // 2. Check existing relationship
        let existing = client.query_opt(
            "SELECT status FROM contacts WHERE user_id = $1 AND contact_user_id = $2",
            &[&user_id, &contact_id],
        ).await?;

        if let Some(row) = existing {
            let status: String = row.get(0);
            if status == "pending" {
                return Err("Request already sent".into());
            } else if status == "accepted" {
                return Err("Already friends".into());
            } else if status == "blocked" {
                return Err("User blocked".into());
            }
        }

        // 3. Insert Pending Request
        client.execute(
            "INSERT INTO contacts (user_id, contact_user_id, status, created_at, updated_at) 
             VALUES ($1, $2, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            &[&user_id, &contact_id],
        ).await?;

        Ok("Friend request sent".to_string())
    }

    /// Accept friend request
    pub async fn accept_request(
        pool: &DbPool,
        user_id: i32,      // Me
        contact_id: i32,   // The person who sent request
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut client = pool.get().await?;
        let transaction = client.transaction().await?;

        // 1. Check if they actually sent a request to me
        // Note: The request sender is 'contact_id', receiver is 'user_id'
        // In my schema plan: A sends to B -> Row(A, B, pending)
        // So B (user_id) accepts A (contact_id) -> We look for Row(contact_id, user_id, pending)
        
        let request = transaction.query_opt(
            "SELECT status FROM contacts WHERE user_id = $1 AND contact_user_id = $2",
            &[&contact_id, &user_id]
        ).await?;

        if let Some(row) = request {
            let status: String = row.get(0);
            if status != "pending" {
                 return Err("No pending request from this user".into());
            }
        } else {
             return Err("No request found".into());
        }

        // 2. Update their row to accepted
        transaction.execute(
            "UPDATE contacts SET status = 'accepted', updated_at = CURRENT_TIMESTAMP 
             WHERE user_id = $1 AND contact_user_id = $2",
             &[&contact_id, &user_id]
        ).await?;

        // 3. Insert/Update MY row to accepted (Bidirectional)
        transaction.execute(
            "INSERT INTO contacts (user_id, contact_user_id, status, created_at, updated_at)
             VALUES ($1, $2, 'accepted', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT (user_id, contact_user_id) 
             DO UPDATE SET status = 'accepted', updated_at = CURRENT_TIMESTAMP",
             &[&user_id, &contact_id]
        ).await?;

        transaction.commit().await?;

        Ok("Friend request accepted".to_string())
    }

    /// Get my contacts (friends)
    pub async fn get_contacts(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Vec<Contact>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let rows = client.query(
            "SELECT c.user_id, c.contact_user_id, c.status, c.created_at, c.updated_at,
                    u.username, u.email, u.first_name, u.last_name, u.is_active
             FROM contacts c
             JOIN users u ON c.contact_user_id = u.id
             WHERE c.user_id = $1 AND c.status = 'accepted'",
            &[&user_id]
        ).await?;

        let contacts = rows.iter().map(|row| Contact {
            user_id: row.get(0),
            contact_id: row.get(1),
            status: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
            contact_user: Some(ContactUser {
                username: row.get(5),
                email: row.get(6),
                first_name: row.get(7),
                last_name: row.get(8),
                is_active: row.get(9),
            }),
        }).collect();

        Ok(contacts)
    }

    /// Get pending requests (People who added ME)
    pub async fn get_pending_requests(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Vec<Contact>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        // Look for rows where I am the contact_id and status is pending
        let rows = client.query(
            "SELECT c.user_id, c.contact_user_id, c.status, c.created_at, c.updated_at,
                    u.username, u.email, u.first_name, u.last_name, u.is_active
             FROM contacts c
             JOIN users u ON c.user_id = u.id
             WHERE c.contact_user_id = $1 AND c.status = 'pending'",
            &[&user_id]
        ).await?;

        let requests = rows.iter().map(|row| Contact {
            user_id: row.get(0),     // This is the SENDER
            contact_id: row.get(1),  // This is ME
            status: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
            contact_user: Some(ContactUser {
                username: row.get(5),
                email: row.get(6),
                first_name: row.get(7),
                last_name: row.get(8),
                is_active: row.get(9),
            }),
        }).collect();

        Ok(requests)
    }
}
