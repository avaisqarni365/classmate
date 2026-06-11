CREATE TABLE IF NOT EXISTS whatsapp_group_links (
  id TEXT PRIMARY KEY,
  group_id TEXT NOT NULL UNIQUE REFERENCES whatsapp_groups(id) ON DELETE CASCADE,
  invite_link TEXT NOT NULL,
  external_name TEXT,
  linked_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS whatsapp_consent (
  user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  opted_in INTEGER NOT NULL DEFAULT 0,
  opted_in_at TEXT,
  source TEXT
);

CREATE TABLE IF NOT EXISTS whatsapp_outbound_messages (
  id TEXT PRIMARY KEY,
  group_id TEXT REFERENCES whatsapp_groups(id) ON DELETE SET NULL,
  user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  kind TEXT NOT NULL,
  ref_id TEXT,
  phone TEXT NOT NULL,
  body TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  wa_message_id TEXT,
  error TEXT,
  created_at TEXT NOT NULL,
  sent_at TEXT,
  delivered_at TEXT,
  read_at TEXT
);
