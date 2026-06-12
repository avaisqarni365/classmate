ALTER TABLE whatsapp_outbound_messages ADD COLUMN broadcast_batch_id TEXT;

CREATE INDEX IF NOT EXISTS idx_wa_outbound_batch ON whatsapp_outbound_messages(broadcast_batch_id);

CREATE TABLE IF NOT EXISTS whatsapp_message_status_events (
  id TEXT PRIMARY KEY,
  outbound_id TEXT REFERENCES whatsapp_outbound_messages(id) ON DELETE SET NULL,
  wa_message_id TEXT NOT NULL,
  group_id TEXT REFERENCES whatsapp_groups(id) ON DELETE SET NULL,
  status TEXT NOT NULL,
  event_at TEXT NOT NULL,
  received_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_wa_mse_wa_id ON whatsapp_message_status_events(wa_message_id);
CREATE INDEX IF NOT EXISTS idx_wa_mse_group ON whatsapp_message_status_events(group_id);
CREATE INDEX IF NOT EXISTS idx_wa_mse_received ON whatsapp_message_status_events(received_at);
