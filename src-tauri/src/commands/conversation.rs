use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::models::conversation::ConversationSummary;
use crate::models::message::MessageDetail;
use crate::AppState;

/// Create a new conversation. Returns the conversation ID.
#[tauri::command]
pub fn create_conversation(
    state: State<'_, Arc<AppState>>,
    title: Option<String>,
) -> Result<String, AppError> {
    let t = title.unwrap_or_else(|| "New Conversation".to_string());
    let conv = state.db.create_conversation(&t)?;
    Ok(conv.id)
}

/// Get all messages for a conversation, with attachments and generation results.
#[tauri::command]
pub fn get_conversation_messages(
    state: State<'_, Arc<AppState>>,
    conversation_id: String,
) -> Result<Vec<MessageDetail>, AppError> {
    state.db.get_conversation_messages(&conversation_id)
}

/// Get conversation list ordered by updated_at DESC.
#[tauri::command]
pub fn get_conversations(
    state: State<'_, Arc<AppState>>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ConversationSummary>, AppError> {
    state
        .db
        .get_conversations(limit.unwrap_or(50), offset.unwrap_or(0))
}

/// Delete a conversation and all its data.
#[tauri::command]
pub fn delete_conversation(
    state: State<'_, Arc<AppState>>,
    conversation_id: String,
) -> Result<(), AppError> {
    state.db.delete_conversation(&conversation_id)
}
