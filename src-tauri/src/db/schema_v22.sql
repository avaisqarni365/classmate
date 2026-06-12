CREATE TABLE course_materials_v22 (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'note' CHECK (kind IN ('note', 'link', 'file', 'textbook')),
  content TEXT NOT NULL,
  created_at TEXT NOT NULL
);

INSERT INTO course_materials_v22 (id, course_id, title, kind, content, created_at)
SELECT id, course_id, title, kind, content, created_at FROM course_materials;

DROP TABLE course_materials;

ALTER TABLE course_materials_v22 RENAME TO course_materials;

CREATE INDEX IF NOT EXISTS idx_materials_course ON course_materials(course_id);
