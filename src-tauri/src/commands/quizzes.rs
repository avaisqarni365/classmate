use crate::models::{
    CreateQuizInput, CreateQuizQuestionInput, GradeQuizAttemptInput, Quiz, QuizAttempt,
    QuizAttemptAnswer, QuizAttemptDetail, QuizDetail, QuizQuestion, SubmitQuizInput,
    SubmitQuizResult,
};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;
const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
fn map_quiz_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Quiz> {
    Ok(Quiz {
        id: row.get(0)?,
        course_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        time_limit_minutes: row.get(4)?,
        max_points: row.get(5)?,
        status: row.get(6)?,
        question_count: row.get(7)?,
        attempt_count: row.get(8)?,
        created_at: row.get(9)?,
    })
}
fn map_question_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<QuizQuestion> {
    let options_json: String = row.get(4)?;
    let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
    Ok(QuizQuestion {
        id: row.get(0)?,
        quiz_id: row.get(1)?,
        prompt: row.get(2)?,
        kind: row.get(3)?,
        options,
        correct_index: row.get(5)?,
        correct_text: row.get(6)?,
        points: row.get(7)?,
        sort_order: row.get(8)?,
    })
}
fn map_attempt_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<QuizAttempt> {
    Ok(QuizAttempt {
        id: row.get(0)?,
        quiz_id: row.get(1)?,
        student_name: row.get(2)?,
        score: row.get(3)?,
        max_score: row.get(4)?,
        review_status: row.get(5)?,
        feedback: row.get(6)?,
        submitted_at: row.get(7)?,
    })
}
#[tauri::command]
pub fn list_quizzes(state: State<'_, AppState>, course_id: String) -> Result<Vec<Quiz>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sql = "SELECT q.id, q.course_id, q.title, q.description, q.time_limit_minutes, q.max_points,  q.status, COUNT(DISTINCT qq.id) AS question_count, COUNT(DISTINCT qa.id) AS attempt_count,  q.created_at FROM quizzes q LEFT JOIN quiz_questions qq ON qq.quiz_id = q.id  LEFT JOIN quiz_attempts qa ON qa.quiz_id = q.id WHERE q.course_id = ?1  GROUP BY q.id ORDER BY q.created_at DESC";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![course_id], map_quiz_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
#[tauri::command]
pub fn create_quiz(state: State<'_, AppState>, input: CreateQuizInput) -> Result<Quiz, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO quizzes (id, course_id, title, description, time_limit_minutes, max_points, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 0, 'draft', ?6)",
        params![
            id,
            input.course_id,
            input.title.trim(),
            input.description,
            input.time_limit_minutes,
            now
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(Quiz {
        id,
        course_id: input.course_id,
        title: input.title.trim().to_string(),
        description: input.description,
        time_limit_minutes: input.time_limit_minutes,
        max_points: 0.0,
        status: "draft".into(),
        question_count: 0,
        attempt_count: 0,
        created_at: now,
    })
}
#[tauri::command]
pub fn add_quiz_question(
    state: State<'_, AppState>,
    input: CreateQuizQuestionInput,
) -> Result<QuizQuestion, String> {
    let kind = input
        .kind
        .clone()
        .unwrap_or_else(|| "mcq".into())
        .to_lowercase();
    if kind == "short_answer" {
        if input.prompt.trim().is_empty() {
            return Err("Prompt is required".into());
        }
    } else if input.options.len() < 2 {
        return Err("At least two options required".into());
    } else if input.correct_index < 0 || input.correct_index as usize >= input.options.len() {
        return Err("Invalid correct answer index".into());
    }
    let id = Uuid::new_v4().to_string();
    let options_json =
        serde_json::to_string(&input.options).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sort_order: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM quiz_questions WHERE quiz_id = ?1",
            params![input.quiz_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    conn.execute(
        "INSERT INTO quiz_questions (id, quiz_id, prompt, kind, options_json, correct_index, correct_text, points, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            input.quiz_id,
            input.prompt.trim(),
            kind,
            options_json,
            input.correct_index,
            input.correct_text,
            input.points,
            sort_order
        ],
    )
    .map_err(|e| e.to_string())?;
    let max_points: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(points), 0) FROM quiz_questions WHERE quiz_id = ?1",
            params![input.quiz_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0);
    conn.execute(
        "UPDATE quizzes SET max_points = ?1 WHERE id = ?2",
        params![max_points, input.quiz_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(QuizQuestion {
        id,
        quiz_id: input.quiz_id,
        prompt: input.prompt.trim().to_string(),
        kind,
        options: input.options,
        correct_index: input.correct_index,
        correct_text: input.correct_text,
        points: input.points,
        sort_order,
    })
}
#[tauri::command]
pub fn publish_quiz(state: State<'_, AppState>, quiz_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM quiz_questions WHERE quiz_id = ?1",
            params![quiz_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if count == 0 {
        return Err("Add at least one question before publishing".into());
    }
    conn.execute(
        "UPDATE quizzes SET status = 'published' WHERE id = ?1",
        params![quiz_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn get_quiz_detail(state: State<'_, AppState>, quiz_id: String) -> Result<QuizDetail, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sql = "SELECT q.id, q.course_id, q.title, q.description, q.time_limit_minutes, q.max_points,  q.status, COUNT(DISTINCT qq.id), COUNT(DISTINCT qa.id), q.created_at FROM quizzes q  LEFT JOIN quiz_questions qq ON qq.quiz_id = q.id LEFT JOIN quiz_attempts qa ON qa.quiz_id = q.id  WHERE q.id = ?1 GROUP BY q.id";
    let quiz = conn
        .query_row(sql, params![quiz_id], map_quiz_row)
        .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, quiz_id, prompt, kind, options_json, correct_index, correct_text, points, sort_order FROM quiz_questions WHERE quiz_id = ?1 ORDER BY sort_order ASC",
        )
        .map_err(|e| e.to_string())?;
    let questions = stmt
        .query_map(params![quiz_id], map_question_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(QuizDetail { quiz, questions })
}
#[tauri::command]
pub fn list_quiz_attempts(
    state: State<'_, AppState>,
    quiz_id: String,
) -> Result<Vec<QuizAttempt>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, quiz_id, student_name, score, max_score, review_status, feedback, submitted_at FROM quiz_attempts WHERE quiz_id = ?1 ORDER BY submitted_at DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![quiz_id], map_attempt_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
#[tauri::command]
pub fn get_quiz_attempt_detail(
    state: State<'_, AppState>,
    attempt_id: String,
) -> Result<QuizAttemptDetail, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let attempt = conn
        .query_row(
            "SELECT id, quiz_id, student_name, score, max_score, review_status, feedback, submitted_at FROM quiz_attempts WHERE id = ?1",
            params![attempt_id],
            map_attempt_row,
        )
        .map_err(|_| "Attempt not found".to_string())?;
    let answers_json: Option<String> = conn
        .query_row(
            "SELECT answers_json FROM quiz_attempts WHERE id = ?1",
            params![attempt_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let answers: Vec<QuizAttemptAnswer> = answers_json
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();
    let mut stmt = conn
        .prepare(
            "SELECT id, quiz_id, prompt, kind, options_json, correct_index, correct_text, points, sort_order FROM quiz_questions WHERE quiz_id = ?1 ORDER BY sort_order ASC",
        )
        .map_err(|e| e.to_string())?;
    let questions = stmt
        .query_map(params![attempt.quiz_id], map_question_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(QuizAttemptDetail {
        attempt,
        questions,
        answers,
    })
}
#[tauri::command]
pub fn grade_quiz_attempt(
    state: State<'_, AppState>,
    input: GradeQuizAttemptInput,
) -> Result<QuizAttempt, String> {
    if input.score < 0.0 {
        return Err("Score cannot be negative".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let max_score: f64 = conn
        .query_row(
            "SELECT max_score FROM quiz_attempts WHERE id = ?1",
            params![input.attempt_id],
            |row| row.get(0),
        )
        .map_err(|_| "Attempt not found".to_string())?;
    if input.score > max_score {
        return Err(format!("Score cannot exceed {:.1}", max_score));
    }
    conn.execute(
        "UPDATE quiz_attempts SET score = ?1, feedback = ?2, review_status = 'complete' WHERE id = ?3",
        params![input.score, input.feedback, input.attempt_id],
    )
    .map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, quiz_id, student_name, score, max_score, review_status, feedback, submitted_at FROM quiz_attempts WHERE id = ?1",
        params![input.attempt_id],
        map_attempt_row,
    )
    .map_err(|e| e.to_string())
}
pub fn grade_quiz_submission(
    conn: &rusqlite::Connection,
    input: SubmitQuizInput,
) -> Result<SubmitQuizResult, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, correct_index, correct_text, points FROM quiz_questions WHERE quiz_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let questions: Vec<(String, String, i64, Option<String>, f64)> = stmt
        .query_map(params![input.quiz_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if questions.is_empty() {
        return Err("Quiz not found".into());
    }
    let mut score = 0.0;
    let mut max_score = 0.0;
    let mut needs_review = false;
    for (_, _, _, _, points) in &questions {
        max_score += points;
    }
    for answer in &input.answers {
        if let Some((_, kind, correct_index, correct_text, points)) = questions
            .iter()
            .find(|(id, _, _, _, _)| id == &answer.question_id)
        {
            match kind.as_str() {
                "short_answer" => {
                    let given = answer.text_answer.as_deref().unwrap_or("").trim();
                    if given.is_empty() {
                        needs_review = true;
                        continue;
                    }
                    if let Some(expected) = correct_text {
                        if given.eq_ignore_ascii_case(expected.trim()) {
                            score += points;
                        } else {
                            needs_review = true;
                        }
                    } else {
                        needs_review = true;
                    }
                }
                _ => {
                    if answer.selected_index == Some(*correct_index) {
                        score += points;
                    }
                }
            }
        }
    }
    let review_status = if needs_review { "pending" } else { "complete" };
    let percent = if max_score > 0.0 {
        (score / max_score) * 100.0
    } else {
        0.0
    };
    let now = Utc::now().to_rfc3339();
    let answers_json = serde_json::to_string(&input.answers).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO quiz_attempts (id, quiz_id, student_name, score, max_score, answers_json, review_status, submitted_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            Uuid::new_v4().to_string(),
            input.quiz_id,
            input.student_name.trim(),
            score,
            max_score,
            answers_json,
            review_status,
            now
        ],
    )
    .map_err(|e| e.to_string())?;
    let message = if needs_review {
        format!(
            "Auto score: {:.1}/{:.1} ({:.0}%) — teacher review pending",
            score, max_score, percent
        )
    } else {
        format!("Score: {:.1}/{:.1} ({:.0}%)", score, max_score, percent)
    };
    Ok(SubmitQuizResult {
        score,
        max_score,
        percent,
        message,
    })
}
#[tauri::command]
pub fn submit_quiz(
    state: State<'_, AppState>,
    input: SubmitQuizInput,
) -> Result<SubmitQuizResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    grade_quiz_submission(&conn, input)
}
#[allow(dead_code)]
pub const MAX_SUBMISSION_FILE_BYTES: usize = MAX_FILE_BYTES;
