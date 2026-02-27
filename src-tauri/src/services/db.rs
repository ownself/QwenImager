use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::conversation::{Conversation, ConversationSummary};
use crate::models::message::{
    Attachment, AttachmentInfo, GenerationResult, GenerationResultInfo, Message, MessageDetail,
};

/// Row tuple from the messages table query.
type MessageRow = (String, String, Option<String>, String, Option<String>, i64);

/// Thread-safe wrapper around a SQLite connection.
/// Used as Tauri managed state via `AppState`.
pub struct Database {
    conn: Mutex<Connection>,
}

/// Get current Unix timestamp in seconds.
fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

impl Database {
    /// Open (or create) the database at the given path and run migrations.
    pub fn new(db_path: &PathBuf) -> Result<Self, AppError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path).map_err(|e| {
            AppError::Io(format!(
                "Failed to open database at {}: {}",
                db_path.display(),
                e
            ))
        })?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Enable foreign key enforcement
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Execute the initial migration SQL to create tables and indexes.
    fn run_migrations(&self) -> Result<(), AppError> {
        let migration_sql = include_str!("../../migrations/001_initial.sql");
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Io(format!("Failed to acquire database lock: {}", e)))?;
        conn.execute_batch(migration_sql)?;
        Ok(())
    }

    /// Acquire a lock on the database connection and execute a closure.
    /// All database operations should go through this method.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Io(format!("Failed to acquire database lock: {}", e)))?;
        f(&conn)
    }

    // ── Conversation CRUD ──

    /// Create a new conversation with the given title. Returns the new Conversation.
    pub fn create_conversation(&self, title: &str) -> Result<Conversation, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let ts = now_ts();
        let effective_title = if title.is_empty() {
            "New Conversation"
        } else {
            title
        };

        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, effective_title, ts, ts],
            )?;
            Ok(Conversation {
                id,
                title: effective_title.to_string(),
                created_at: ts,
                updated_at: ts,
            })
        })
    }

    /// Update a conversation's title.
    pub fn update_conversation_title(
        &self,
        conversation_id: &str,
        title: &str,
    ) -> Result<(), AppError> {
        self.with_conn(|conn| {
            let rows = conn.execute(
                "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![title, now_ts(), conversation_id],
            )?;
            if rows == 0 {
                return Err(AppError::NotFound(format!(
                    "Conversation not found: {}",
                    conversation_id
                )));
            }
            Ok(())
        })
    }

    /// Touch the conversation's updated_at timestamp.
    fn touch_conversation(&self, conn: &Connection, conversation_id: &str) -> Result<(), AppError> {
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![now_ts(), conversation_id],
        )?;
        Ok(())
    }

    /// Get conversations ordered by updated_at DESC, with message counts.
    pub fn get_conversations(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ConversationSummary>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT c.id, c.title, c.created_at, c.updated_at,
                        (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id) AS msg_count
                 FROM conversations c
                 ORDER BY c.updated_at DESC
                 LIMIT ?1 OFFSET ?2",
            )?;
            let rows = stmt.query_map(params![limit, offset], |row| {
                Ok(ConversationSummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    message_count: row.get(4)?,
                })
            })?;
            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok(result)
        })
    }

    /// Delete a conversation and all its messages, attachments, and results (via CASCADE).
    pub fn delete_conversation(&self, conversation_id: &str) -> Result<(), AppError> {
        self.with_conn(|conn| {
            let rows = conn.execute(
                "DELETE FROM conversations WHERE id = ?1",
                params![conversation_id],
            )?;
            if rows == 0 {
                return Err(AppError::NotFound(format!(
                    "Conversation not found: {}",
                    conversation_id
                )));
            }
            Ok(())
        })
    }

    // ── Message CRUD ──

    /// Add a message to a conversation. Returns the created Message.
    /// If this is the first user message, updates the conversation title
    /// with the first 30 characters of the text content.
    pub fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        text_content: Option<&str>,
        mode: &str,
        extra_params: Option<&str>,
    ) -> Result<Message, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let ts = now_ts();

        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO messages (id, conversation_id, role, text_content, mode, extra_params, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, conversation_id, role, text_content, mode, extra_params, ts],
            )?;

            // Auto-update conversation title from first user message
            if role == "user" {
                if let Some(text) = text_content {
                    // Check if this is the first user message in the conversation
                    let msg_count: i64 = conn.query_row(
                        "SELECT COUNT(*) FROM messages WHERE conversation_id = ?1 AND role = 'user'",
                        params![conversation_id],
                        |row| row.get(0),
                    )?;
                    if msg_count == 1 {
                        // First user message — update title with first 30 chars
                        let title: String = text.chars().take(30).collect();
                        conn.execute(
                            "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
                            params![title, ts, conversation_id],
                        )?;
                    } else {
                        self.touch_conversation(conn, conversation_id)?;
                    }
                } else {
                    self.touch_conversation(conn, conversation_id)?;
                }
            } else {
                self.touch_conversation(conn, conversation_id)?;
            }

            Ok(Message {
                id,
                conversation_id: conversation_id.to_string(),
                role: role.to_string(),
                text_content: text_content.map(|s| s.to_string()),
                mode: mode.to_string(),
                extra_params: extra_params.map(|s| s.to_string()),
                created_at: ts,
            })
        })
    }

    /// Add a generation result linked to a message.
    pub fn add_generation_result(
        &self,
        message_id: &str,
        resource_url: Option<&str>,
        local_path: Option<&str>,
        resource_type: &str,
        model_used: &str,
        generation_params: Option<&str>,
    ) -> Result<GenerationResult, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let ts = now_ts();

        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO generation_results (id, message_id, resource_url, local_path, resource_type, model_used, generation_params, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, message_id, resource_url, local_path, resource_type, model_used, generation_params, ts],
            )?;
            Ok(GenerationResult {
                id,
                message_id: message_id.to_string(),
                resource_url: resource_url.map(|s| s.to_string()),
                local_path: local_path.map(|s| s.to_string()),
                resource_type: resource_type.to_string(),
                model_used: model_used.to_string(),
                generation_params: generation_params.map(|s| s.to_string()),
                created_at: ts,
            })
        })
    }

    // ── Attachment CRUD ──

    /// Add an attachment to a message.
    pub fn add_attachment(
        &self,
        message_id: &str,
        file_path: &str,
        display_order: i32,
        file_size: i64,
        mime_type: &str,
        source: &str,
    ) -> Result<Attachment, AppError> {
        let id = uuid::Uuid::new_v4().to_string();

        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO attachments (id, message_id, file_path, display_order, file_size, mime_type, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, message_id, file_path, display_order, file_size, mime_type, source],
            )?;
            Ok(Attachment {
                id,
                message_id: message_id.to_string(),
                file_path: file_path.to_string(),
                display_order,
                file_size,
                mime_type: mime_type.to_string(),
                source: source.to_string(),
            })
        })
    }

    /// Get all attachments for a message, ordered by display_order.
    pub fn get_attachments_by_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<AttachmentInfo>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, file_path, display_order, file_size, mime_type, source
                 FROM attachments
                 WHERE message_id = ?1
                 ORDER BY display_order ASC",
            )?;
            let rows = stmt
                .query_map(params![message_id], |row| {
                    Ok(AttachmentInfo {
                        id: row.get(0)?,
                        file_path: row.get(1)?,
                        display_order: row.get(2)?,
                        file_size: row.get(3)?,
                        mime_type: row.get(4)?,
                        source: row.get(5)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    /// Get all messages for a conversation, with their attachments and generation results.
    /// Messages are ordered by created_at ASC.
    pub fn get_conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<MessageDetail>, AppError> {
        self.with_conn(|conn| {
            // First, get all messages
            let mut msg_stmt = conn.prepare(
                "SELECT id, role, text_content, mode, extra_params, created_at
                 FROM messages
                 WHERE conversation_id = ?1
                 ORDER BY created_at ASC",
            )?;
            let messages: Vec<MessageRow> = msg_stmt
                .query_map(params![conversation_id], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut result = Vec::with_capacity(messages.len());

            // For each message, fetch attachments and generation results
            let mut att_stmt = conn.prepare(
                "SELECT id, file_path, display_order, file_size, mime_type, source
                 FROM attachments
                 WHERE message_id = ?1
                 ORDER BY display_order ASC",
            )?;
            let mut res_stmt = conn.prepare(
                "SELECT id, resource_url, local_path, resource_type, model_used, created_at
                 FROM generation_results
                 WHERE message_id = ?1",
            )?;

            for (msg_id, role, text_content, mode, extra_params, created_at) in &messages {
                let attachments: Vec<AttachmentInfo> = att_stmt
                    .query_map(params![msg_id], |row| {
                        Ok(AttachmentInfo {
                            id: row.get(0)?,
                            file_path: row.get(1)?,
                            display_order: row.get(2)?,
                            file_size: row.get(3)?,
                            mime_type: row.get(4)?,
                            source: row.get(5)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                let results: Vec<GenerationResultInfo> = res_stmt
                    .query_map(params![msg_id], |row| {
                        Ok(GenerationResultInfo {
                            id: row.get(0)?,
                            resource_url: row.get(1)?,
                            local_path: row.get(2)?,
                            resource_type: row.get(3)?,
                            model_used: row.get(4)?,
                            created_at: row.get(5)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                let extra_params_value: Option<serde_json::Value> = extra_params
                    .as_ref()
                    .and_then(|s| serde_json::from_str(s).ok());

                result.push(MessageDetail {
                    id: msg_id.clone(),
                    role: role.clone(),
                    text_content: text_content.clone(),
                    mode: mode.clone(),
                    attachments,
                    results,
                    extra_params: extra_params_value,
                    created_at: *created_at,
                });
            }

            Ok(result)
        })
    }
}

/// Resolve the database file path: `{app_data_dir}/qwenimager.db`.
pub fn resolve_db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("qwenimager.db")
}
