CREATE TABLE IF NOT EXISTS push_devices (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  platform TEXT NOT NULL CHECK (platform IN ('fcm', 'apns')),
  token TEXT NOT NULL,
  device_name TEXT,
  created_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  UNIQUE (platform, token)
);

CREATE TABLE IF NOT EXISTS push_log (
  id TEXT PRIMARY KEY,
  user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  platform TEXT NOT NULL,
  token TEXT NOT NULL,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('sent', 'failed')),
  provider_response TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_push_devices_user ON push_devices(user_id);
CREATE INDEX IF NOT EXISTS idx_push_log_created ON push_log(created_at DESC);
