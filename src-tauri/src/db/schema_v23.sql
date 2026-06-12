CREATE TABLE course_materials_v23 (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'note' CHECK (kind IN ('note', 'link', 'file', 'textbook', 'speak_note', 'handwriting')),
  content TEXT NOT NULL,
  created_at TEXT NOT NULL
);

INSERT INTO course_materials_v23 (id, course_id, title, kind, content, created_at)
SELECT id, course_id, title, kind, content, created_at FROM course_materials;

DROP TABLE course_materials;

ALTER TABLE course_materials_v23 RENAME TO course_materials;

CREATE INDEX IF NOT EXISTS idx_materials_course ON course_materials(course_id);

CREATE TABLE IF NOT EXISTS note_capture_sessions (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  created_by TEXT NOT NULL REFERENCES users(id),
  title TEXT NOT NULL,
  ink_json TEXT NOT NULL DEFAULT '[]',
  preview_data_url TEXT,
  status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'attached', 'expired')),
  material_id TEXT REFERENCES course_materials(id) ON DELETE SET NULL,
  expires_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_capture_course ON note_capture_sessions(course_id);

CREATE TABLE IF NOT EXISTS material_lab_completions (
  id TEXT PRIMARY KEY,
  material_id TEXT NOT NULL REFERENCES course_materials(id) ON DELETE CASCADE,
  student_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  completed_at TEXT NOT NULL,
  UNIQUE(material_id, student_id)
);

CREATE INDEX IF NOT EXISTS idx_lab_complete_material ON material_lab_completions(material_id);
