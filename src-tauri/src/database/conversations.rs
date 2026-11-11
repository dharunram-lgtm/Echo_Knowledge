use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub citations_json: Option<String>,
    pub created_at: String,
}

pub fn create_conversation(conn: &Connection, title: Option<&str>) -> Result<Conversation> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let conv_title = title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or("New Research Chat")
        .to_string();

    conn.execute(
        "INSERT INTO conversations (id, title, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4);",
        params![id, conv_title, now, now],
    )?;

    Ok(Conversation {
        id,
        title: conv_title,
        created_at: now.clone(),
        updated_at: now,
        message_count: 0,
    })
}

pub fn list_conversations(conn: &Connection) -> Result<Vec<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.created_at, c.updated_at, COUNT(m.id) as msg_count
         FROM conversations c
         LEFT JOIN conversation_messages m ON c.id = m.conversation_id
         GROUP BY c.id
         ORDER BY c.updated_at DESC;",
    )?;

    let rows = stmt.query_map([], |row| {
        let count: i64 = row.get(4)?;
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            message_count: count as usize,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn get_conversation(conn: &Connection, id: &str) -> Result<Option<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.created_at, c.updated_at, COUNT(m.id)
         FROM conversations c
         LEFT JOIN conversation_messages m ON c.id = m.conversation_id
         WHERE c.id = ?1
         GROUP BY c.id;",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        let count: i64 = row.get(4)?;
        Ok(Some(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            message_count: count as usize,
        }))
    } else {
        Ok(None)
    }
}

pub fn update_conversation_title(conn: &Connection, id: &str, title: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3;",
        params![title, now, id],
    )?;
    Ok(())
}

pub fn delete_conversation(conn: &Connection, id: &str) -> Result<bool> {
    let affected = conn.execute("DELETE FROM conversations WHERE id = ?1;", params![id])?;
    Ok(affected > 0)
}

pub fn clear_conversation_messages(conn: &Connection, id: &str) -> Result<bool> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "DELETE FROM conversation_messages WHERE conversation_id = ?1;",
        params![id],
    )?;
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2;",
        params![now, id],
    )?;
    Ok(true)
}

pub fn add_message(
    conn: &Connection,
    conversation_id: &str,
    role: &str,
    content: &str,
    citations_json: Option<&str>,
) -> Result<ConversationMessage> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO conversation_messages (id, conversation_id, role, content, citations_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
        params![id, conversation_id, role, content, citations_json, now],
    )?;

    // Touch conversation updated_at
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2;",
        params![now, conversation_id],
    )?;

    Ok(ConversationMessage {
        id,
        conversation_id: conversation_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        citations_json: citations_json.map(|s| s.to_string()),
        created_at: now,
    })
}

pub fn get_conversation_messages(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<ConversationMessage>> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, citations_json, created_at
         FROM conversation_messages
         WHERE conversation_id = ?1
         ORDER BY created_at ASC;",
    )?;

    let rows = stmt.query_map(params![conversation_id], |row| {
        Ok(ConversationMessage {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            citations_json: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    let mut msgs = Vec::new();
    for row in rows {
        msgs.push(row?);
    }
    Ok(msgs)
}

