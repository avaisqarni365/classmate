ALTER TABLE courses ADD COLUMN school_id TEXT REFERENCES schools(id);

CREATE TABLE IF NOT EXISTS school_members (
  id TEXT PRIMARY KEY,
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  UNIQUE (school_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_courses_school ON courses(school_id);
CREATE INDEX IF NOT EXISTS idx_school_members_school ON school_members(school_id);
CREATE INDEX IF NOT EXISTS idx_school_members_user ON school_members(user_id);
