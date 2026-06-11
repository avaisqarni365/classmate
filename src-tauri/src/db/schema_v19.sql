CREATE TABLE IF NOT EXISTS whatsapp_group_participant_events (
  id TEXT PRIMARY KEY,
  group_id TEXT REFERENCES whatsapp_groups(id) ON DELETE SET NULL,
  external_group_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  direction TEXT,
  wa_id TEXT,
  reason TEXT,
  join_request_id TEXT,
  received_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_wa_gpe_group ON whatsapp_group_participant_events(group_id);
CREATE INDEX IF NOT EXISTS idx_wa_gpe_external ON whatsapp_group_participant_events(external_group_id);
CREATE INDEX IF NOT EXISTS idx_wa_gpe_received ON whatsapp_group_participant_events(received_at);

ALTER TABLE whatsapp_group_links ADD COLUMN cached_participants_json TEXT;
ALTER TABLE whatsapp_group_links ADD COLUMN roster_synced_at TEXT;
