CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('admin', 'teacher', 'student', 'parent')),
  password_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS courses (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  code TEXT NOT NULL,
  description TEXT,
  teacher_id TEXT REFERENCES users(id),
  term TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS enrollments (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  student_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  status TEXT NOT NULL DEFAULT 'active',
  enrolled_at TEXT NOT NULL,
  UNIQUE (course_id, student_id)
);

CREATE TABLE IF NOT EXISTS assignments (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  description TEXT,
  due_at TEXT,
  max_points REAL NOT NULL DEFAULT 100,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS grades (
  id TEXT PRIMARY KEY,
  assignment_id TEXT NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
  student_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  points REAL,
  feedback TEXT,
  graded_at TEXT,
  UNIQUE (assignment_id, student_id)
);

CREATE TABLE IF NOT EXISTS class_sessions (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  pin TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'scheduled',
  started_at TEXT,
  ended_at TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS attendance (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES class_sessions(id) ON DELETE CASCADE,
  student_name TEXT NOT NULL,
  student_id TEXT REFERENCES users(id),
  joined_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_courses_teacher ON courses(teacher_id);
CREATE INDEX IF NOT EXISTS idx_enrollments_course ON enrollments(course_id);
CREATE INDEX IF NOT EXISTS idx_assignments_course ON assignments(course_id);
CREATE TABLE IF NOT EXISTS course_materials (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'note' CHECK (kind IN ('note', 'link', 'file', 'textbook', 'speak_note', 'handwriting')),
  content TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_course ON class_sessions(course_id);
CREATE INDEX IF NOT EXISTS idx_materials_course ON course_materials(course_id);
