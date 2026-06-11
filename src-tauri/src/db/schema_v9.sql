ALTER TABLE users ADD COLUMN phone TEXT;

CREATE TABLE IF NOT EXISTS whatsapp_groups (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'students' CHECK (kind IN ('students', 'teachers', 'custom')),
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS whatsapp_group_members (
  id TEXT PRIMARY KEY,
  group_id TEXT NOT NULL REFERENCES whatsapp_groups(id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  UNIQUE (group_id, user_id)
);
