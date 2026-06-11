CREATE TABLE IF NOT EXISTS assignment_rubrics (
  assignment_id TEXT PRIMARY KEY REFERENCES assignments(id) ON DELETE CASCADE,
  criteria_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

ALTER TABLE grades ADD COLUMN rubric_scores_json TEXT;
