CREATE TABLE IF NOT EXISTS schedule_slots (
  id TEXT PRIMARY KEY,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
  start_time TEXT NOT NULL,
  end_time TEXT NOT NULL,
  room TEXT,
  title TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS session_polls (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES class_sessions(id) ON DELETE CASCADE,
  question TEXT NOT NULL,
  options_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS session_poll_votes (
  id TEXT PRIMARY KEY,
  poll_id TEXT NOT NULL REFERENCES session_polls(id) ON DELETE CASCADE,
  student_name TEXT NOT NULL,
  option_index INTEGER NOT NULL,
  voted_at TEXT NOT NULL,
  UNIQUE (poll_id, student_name)
);

CREATE TABLE IF NOT EXISTS assignment_submissions (
  id TEXT PRIMARY KEY,
  assignment_id TEXT NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
  student_name TEXT NOT NULL,
  student_id TEXT REFERENCES users(id),
  body TEXT NOT NULL,
  points REAL,
  feedback TEXT,
  status TEXT NOT NULL DEFAULT 'submitted' CHECK (status IN ('submitted', 'graded')),
  submitted_at TEXT NOT NULL,
  graded_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_schedule_course ON schedule_slots(course_id);
CREATE INDEX IF NOT EXISTS idx_session_polls_session ON session_polls(session_id);
CREATE INDEX IF NOT EXISTS idx_poll_votes_poll ON session_poll_votes(poll_id);
CREATE INDEX IF NOT EXISTS idx_submissions_assignment ON assignment_submissions(assignment_id);
