CREATE TABLE IF NOT EXISTS whatsapp_scheduled_broadcasts (
  id TEXT PRIMARY KEY,
  group_id TEXT NOT NULL REFERENCES whatsapp_groups(id) ON DELETE CASCADE,
  broadcast_kind TEXT NOT NULL,
  kind TEXT,
  assignment_id TEXT,
  announcement_id TEXT,
  custom_message TEXT,
  scheduled_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  sent_at TEXT,
  result_sent INTEGER,
  result_failed INTEGER,
  result_skipped INTEGER,
  error TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_whatsapp_scheduled_due ON whatsapp_scheduled_broadcasts(status, scheduled_at);
