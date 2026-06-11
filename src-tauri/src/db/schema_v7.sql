ALTER TABLE quiz_attempts ADD COLUMN review_status TEXT NOT NULL DEFAULT 'complete';
ALTER TABLE quiz_attempts ADD COLUMN feedback TEXT;
ALTER TABLE assignment_submissions ADD COLUMN file_name TEXT;
ALTER TABLE assignment_submissions ADD COLUMN file_data TEXT;
