CREATE TABLE IF NOT EXISTS cashbook_entries (
  id TEXT PRIMARY KEY,
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  direction TEXT NOT NULL,
  category TEXT NOT NULL,
  amount REAL NOT NULL,
  currency TEXT NOT NULL DEFAULT 'USD',
  description TEXT,
  user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  course_id TEXT REFERENCES courses(id) ON DELETE SET NULL,
  payment_method TEXT NOT NULL DEFAULT 'cash',
  reference TEXT,
  entry_date TEXT NOT NULL,
  created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cashbook_entries_school ON cashbook_entries(school_id);
CREATE INDEX IF NOT EXISTS idx_cashbook_entries_date ON cashbook_entries(entry_date);
CREATE INDEX IF NOT EXISTS idx_cashbook_entries_user ON cashbook_entries(user_id);
