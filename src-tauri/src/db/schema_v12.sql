CREATE TABLE IF NOT EXISTS whatsapp_inbound_messages (
  id TEXT PRIMARY KEY,
  wa_message_id TEXT NOT NULL UNIQUE,
  from_phone TEXT NOT NULL,
  from_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  from_user_name TEXT,
  body TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  routed_topic_id TEXT REFERENCES forum_topics(id) ON DELETE SET NULL,
  routed_post_id TEXT REFERENCES forum_posts(id) ON DELETE SET NULL,
  received_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_inbound_status ON whatsapp_inbound_messages(status);
CREATE INDEX IF NOT EXISTS idx_whatsapp_inbound_received ON whatsapp_inbound_messages(received_at);
