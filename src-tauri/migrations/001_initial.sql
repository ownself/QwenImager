-- QwenImager initial schema
-- Matches data-model.md entity definitions

CREATE TABLE IF NOT EXISTS conversations (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL CHECK(length(title) <= 100),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_conversations_updated ON conversations(updated_at DESC);

CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
    text_content    TEXT,
    mode            TEXT NOT NULL CHECK(mode IN ('text2img', 'img2img', 'translate')),
    extra_params    TEXT,
    created_at      INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id, created_at);

CREATE TABLE IF NOT EXISTS attachments (
    id            TEXT PRIMARY KEY,
    message_id    TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_path     TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    file_size     INTEGER NOT NULL CHECK(file_size <= 10485760),
    mime_type     TEXT NOT NULL CHECK(mime_type IN ('image/png','image/jpeg','image/webp','image/gif')),
    source        TEXT NOT NULL CHECK(source IN ('upload', 'clipboard'))
);

CREATE INDEX IF NOT EXISTS idx_attachments_msg ON attachments(message_id, display_order);

CREATE TABLE IF NOT EXISTS generation_results (
    id                TEXT PRIMARY KEY,
    message_id        TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    resource_url      TEXT,
    local_path        TEXT,
    resource_type     TEXT NOT NULL CHECK(resource_type IN ('image', 'video')),
    model_used        TEXT NOT NULL,
    generation_params TEXT,
    created_at        INTEGER NOT NULL,
    CHECK(resource_url IS NOT NULL OR local_path IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_results_msg ON generation_results(message_id);
