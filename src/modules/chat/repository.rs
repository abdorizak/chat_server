use crate::db::DbPool;
use crate::modules::chat::model::{Message, Group};

pub struct MessageRepository;

impl MessageRepository {
    /// Create or get existing conversation between two users
    pub async fn get_or_create_conversation(
        pool: &DbPool,
        user1_id: i32,
        user2_id: i32,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let client = pool.get().await?;
        
        // Ensure smaller ID is first to enforce uniqueness constraint
        let (p1, p2) = if user1_id < user2_id {
            (user1_id, user2_id)
        } else {
            (user2_id, user1_id)
        };

        // Try to find existing conversation
        let row = client.query_opt(
            "SELECT id FROM conversations WHERE participant_1 = $1 AND participant_2 = $2",
            &[&p1, &p2],
        ).await?;

        if let Some(r) = row {
            return Ok(r.get(0));
        }

        // Create new conversation
        let row = client.query_one(
            "INSERT INTO conversations (participant_1, participant_2) VALUES ($1, $2) RETURNING id",
            &[&p1, &p2]
        ).await?;

        Ok(row.get(0))
    }

    /// Save a new message to database
    pub async fn create_message(
        pool: &DbPool,
        sender_id: i32,
        recipient_id: i32,
        content: &str,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        // 1. Get Conversation ID
        let conversation_id = Self::get_or_create_conversation(pool, sender_id, recipient_id).await?;
        
        let client = pool.get().await?;

        // 2. Insert Message
        let row = client.query_one(
            "INSERT INTO messages (conversation_id, sender_id, content, message_type) 
             VALUES ($1, $2, $3, 'text') 
             RETURNING id, conversation_id, sender_id, content, message_type, sent_at, read_at",
            &[&conversation_id, &sender_id, &content]
        ).await?;

        Ok(Message {
            id: row.get(0),
            conversation_id: row.get(1),
            sender_id: row.get(2),
            content: row.get(3),
            message_type: row.get(4),
            sent_at: row.get(5),
            read_at: row.get(6),
        })
    }
    /// Get message history between two users
    pub async fn get_messages(
        pool: &DbPool,
        user1_id: i32,
        user2_id: i32,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        // 1. Get conversation ID
        // We do a readonly lookup here. If no conversation exists, empty list.
        // Ensure order for lookup
        let (p1, p2) = if user1_id < user2_id { (user1_id, user2_id) } else { (user2_id, user1_id) };
        
        let conv_row = client.query_opt(
            "SELECT id FROM conversations WHERE participant_1 = $1 AND participant_2 = $2",
            &[&p1, &p2]
        ).await?;

        let conversation_id: i32 = match conv_row {
            Some(row) => row.get(0),
            None => return Ok(vec![]),
        };

        // 2. Fetch messages
        let rows = client.query(
            "SELECT id, conversation_id, sender_id, content, message_type, sent_at, read_at
             FROM messages
             WHERE conversation_id = $1
             ORDER BY sent_at DESC
             LIMIT $2 OFFSET $3",
            &[&conversation_id, &limit, &offset]
        ).await?;

        let messages = rows.iter().map(|row| Message {
            id: row.get(0),
            conversation_id: row.get(1),
            sender_id: row.get(2),
            content: row.get(3),
            message_type: row.get(4),
            sent_at: row.get(5),
            read_at: row.get(6),
        }).collect();

        Ok(messages)
        }

    /// Create a new group with initial members
    pub async fn create_group(
        pool: &DbPool,
        creator_id: i32,
        name: &str,
        description: Option<String>,
        member_ids: Vec<i32>,
    ) -> Result<Group, Box<dyn std::error::Error>> {
        let mut client = pool.get().await?;
        let transaction = client.transaction().await?;

        // 1. Create Group
        let group_row = transaction.query_one(
            "INSERT INTO groups (name, description, created_by) VALUES ($1, $2, $3) 
             RETURNING id, name, description, created_by, created_at",
            &[&name, &description, &creator_id]
        ).await?;
        
        let group_id: i32 = group_row.get(0);

        // 2. Add Creator as Admin
        transaction.execute(
            "INSERT INTO group_members (group_id, user_id, role) VALUES ($1, $2, 'admin')",
            &[&group_id, &creator_id]
        ).await?;

        // 3. Add other members
        for member_id in member_ids {
            // Only add different users
            if member_id != creator_id {
                 transaction.execute(
                    "INSERT INTO group_members (group_id, user_id, role) VALUES ($1, $2, 'member')
                     ON CONFLICT DO NOTHING", // Skip duplicates
                    &[&group_id, &member_id]
                ).await?;
            }
        }

        transaction.commit().await?;

        Ok(Group {
            id: group_id,
            name: group_row.get(1),
            description: group_row.get(2),
            creator_id: group_row.get(3),
            created_at: group_row.get(4),
        })
    }

    /// Get user's groups
    pub async fn get_user_groups(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Vec<Group>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let rows = client.query(
            "SELECT g.id, g.name, g.description, g.created_by, g.created_at
             FROM groups g
             JOIN group_members gm ON g.id = gm.group_id
             WHERE gm.user_id = $1
             ORDER BY g.created_at DESC",
             &[&user_id]
        ).await?;

        let groups = rows.iter().map(|row| Group {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            creator_id: row.get(3),
            created_at: row.get(4),
        }).collect();

        Ok(groups)
    }

    /// Save a new group message
    pub async fn create_group_message(
        pool: &DbPool,
        sender_id: i32,
        group_id: i32,
        content: &str,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        // Check if sender is a member (basic security)
        let member_check = client.query_opt(
            "SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2",
            &[&group_id, &sender_id]
        ).await?;

        if member_check.is_none() {
            return Err("User is not a member of this group".into());
        }

        let row = client.query_one(
            "INSERT INTO group_messages (group_id, sender_id, content, message_type)
             VALUES ($1, $2, $3, 'text')
             RETURNING id, group_id, sender_id, content, message_type, sent_at",
            &[&group_id, &sender_id, &content]
        ).await?;

        Ok(Message {
            id: row.get(0),
            conversation_id: 0, // 0 indicates group message in this unified struct, or handle separately
            sender_id: row.get(2),
            content: row.get(3),
            message_type: row.get(4),
            sent_at: row.get(5),
            read_at: None, // Group messages read status is complex (many users), skipping for now
        })
    }

    /// Get all user IDs in a group
    pub async fn get_group_members(
        pool: &DbPool,
        group_id: i32,
    ) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let rows = client.query(
            "SELECT user_id FROM group_members WHERE group_id = $1",
            &[&group_id]
        ).await?;

        Ok(rows.iter().map(|r| r.get(0)).collect())
    }
    /// Get the other participant in a conversation
    pub async fn get_conversation_partner(
        pool: &DbPool,
        conversation_id: i32,
        my_user_id: i32,
    ) -> Result<Option<i32>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        let row = client.query_opt(
            "SELECT participant_1, participant_2 FROM conversations WHERE id = $1",
            &[&conversation_id]
        ).await?;

        if let Some(r) = row {
            let p1: i32 = r.get(0);
            let p2: i32 = r.get(1);
            
            if p1 == my_user_id {
                return Ok(Some(p2));
            } else if p2 == my_user_id {
                return Ok(Some(p1));
            }
        }

        Ok(None)
    }

    /// Mark a message as read and return the sender_id (to notify them)
    pub async fn mark_message_read(
        pool: &DbPool,
        message_id: i32,
        _reader_id: i32, // potentially used for verification
    ) -> Result<Option<i32>, Box<dyn std::error::Error>> {
        let client = pool.get().await?;

        // Update read_at if not already set
        // return sender_id so we can notify them
        let row = client.query_opt(
            "UPDATE messages 
             SET read_at = NOW() 
             WHERE id = $1 AND read_at IS NULL 
             RETURNING sender_id",
            &[&message_id]
        ).await?;

        match row {
            Some(r) => Ok(Some(r.get(0))),
            None => Ok(None) // Already read or not found
        }
    }
}

