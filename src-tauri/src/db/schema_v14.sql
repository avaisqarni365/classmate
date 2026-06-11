CREATE TABLE IF NOT EXISTS whatsapp_consent_log (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  opted_in INTEGER NOT NULL,
  source TEXT NOT NULL,
  note TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_consent_log_user ON whatsapp_consent_log(user_id, created_at);
