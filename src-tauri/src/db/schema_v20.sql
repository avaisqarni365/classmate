CREATE TABLE IF NOT EXISTS whatsapp_group_settings_events (
  id TEXT PRIMARY KEY,
  group_id TEXT REFERENCES whatsapp_groups(id) ON DELETE SET NULL,
  external_group_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  setting_kind TEXT,
  setting_value TEXT,
  update_successful INTEGER,
  error_summary TEXT,
  received_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_wa_gse_group ON whatsapp_group_settings_events(group_id);
CREATE INDEX IF NOT EXISTS idx_wa_gse_external ON whatsapp_group_settings_events(external_group_id);
CREATE INDEX IF NOT EXISTS idx_wa_gse_received ON whatsapp_group_settings_events(received_at);

ALTER TABLE whatsapp_group_links ADD COLUMN group_description TEXT;
ALTER TABLE whatsapp_group_links ADD COLUMN settings_updated_at TEXT;
