ALTER TABLE whatsapp_group_links ADD COLUMN external_group_id TEXT;
ALTER TABLE whatsapp_group_links ADD COLUMN native_status TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE whatsapp_group_links ADD COLUMN creation_error TEXT;
ALTER TABLE whatsapp_group_links ADD COLUMN join_approval_mode TEXT DEFAULT 'auto_approve';

CREATE INDEX IF NOT EXISTS idx_whatsapp_group_links_external ON whatsapp_group_links(external_group_id);
