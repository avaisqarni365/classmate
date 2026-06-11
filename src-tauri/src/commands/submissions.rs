use crate::models::{AssignmentSubmission, GradeSubmissionInput, SubmitAssignmentInput};
use crate::AppState;
use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;
use uuid::Uuid;
const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
fn map_submission_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssignmentSubmission> {
    Ok(AssignmentSubmission {
        id: row.get(0)?,
        assignment_id: row.get(1)?,
        assignment_title: row.get(2)?,
        student_name: row.get(3)?,
        student_id: row.get(4)?,
        body: row.get(5)?,
        file_name: row.get(6)?,
        file_data: row.get(7)?,
        points: row.get(8)?,
        feedback: row.get(9)?,
        status: row.get(10)?,
        submitted_at: row.get(11)?,
        graded_at: row.get(12)?,
    })
}
fn validate_file(file_name: &Option<String>, file_data: &Option<String>) -> Result<(), String> {
    let name = file_name.as_deref().unwrap_or("").trim();
    let data = file_data.as_deref().unwrap_or("").trim();
    if name.is_empty() && data.is_empty() {
        return Ok(());
    }
    if name.is_empty() || data.is_empty() {
        return Err("File name and data are both required for attachments".into());
    }
    let bytes = base64_decode(data)?;
    if bytes.is_empty() {
        return Err("Attached file is empty".into());
    }
    if bytes.len() > MAX_FILE_BYTES {
        return Err("Attached file exceeds 2 MB limit".into());
    }
    Ok(())
}
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let table: [i8; 256] = {
        let mut t = [-1i8; 256];
        for (i, &c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
            t[c as usize] = i as i8;
        }
        t
    };
    let mut out = Vec::with_capacity(cleaned.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;
    for c in cleaned.chars() {
        if c == '=' {
            break;
        }
        let val = table[c as usize];
        if val < 0 {
            return Err("Invalid file encoding".into());
        }
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
#[tauri::command]
pub fn list_submissions(
    state: State<'_, AppState>,
    course_id: String,
) -> Result<Vec<AssignmentSubmission>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT sub.id, sub.assignment_id, a.title, sub.student_name, sub.student_id,
                    sub.body, sub.file_name, sub.file_data, sub.points, sub.feedback, sub.status,
                    sub.submitted_at, sub.graded_at
             FROM assignment_submissions sub
             JOIN assignments a ON a.id = sub.assignment_id
             WHERE a.course_id = ?1
             ORDER BY sub.submitted_at DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![course_id], map_submission_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
pub fn submit_assignment_work(
    conn: &Connection,
    input: SubmitAssignmentInput,
    student_id: Option<String>,
) -> Result<AssignmentSubmission, String> {
    if input.student_name.trim().is_empty() {
        return Err("Student name is required".into());
    }
    let body = input.body.trim();
    let file_name = input
        .file_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let file_data = input
        .file_data
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    validate_file(&file_name, &file_data)?;
    if body.is_empty() && file_name.is_none() {
        return Err("Enter text or attach a file".into());
    }
    let (assignment_title, course_id): (String, String) = conn
        .query_row(
            "SELECT title, course_id FROM assignments WHERE id = ?1",
            params![input.assignment_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Assignment not found".to_string())?;
    let student_id: Option<String> = if let Some(id) = student_id {
        Some(id)
    } else {
        conn.query_row(
            "SELECT u.id FROM users u
             JOIN enrollments e ON e.student_id = u.id
             WHERE e.course_id = ?1 AND e.status = 'active'
               AND LOWER(u.name) = LOWER(?2) AND u.role = 'student'
             LIMIT 1",
            params![course_id, input.student_name.trim()],
            |row| row.get(0),
        )
        .ok()
    };
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO assignment_submissions (id, assignment_id, student_name, student_id, body, file_name, file_data, status, submitted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'submitted', ?8)",
        params![
            id,
            input.assignment_id,
            input.student_name.trim(),
            student_id,
            body,
            file_name,
            file_data,
            now
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(AssignmentSubmission {
        id,
        assignment_id: input.assignment_id,
        assignment_title,
        student_name: input.student_name.trim().to_string(),
        student_id,
        body: body.to_string(),
        file_name,
        file_data,
        points: None,
        feedback: None,
        status: "submitted".into(),
        submitted_at: now,
        graded_at: None,
    })
}
#[tauri::command]
pub fn submit_assignment(
    state: State<'_, AppState>,
    input: SubmitAssignmentInput,
) -> Result<AssignmentSubmission, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    submit_assignment_work(&conn, input, None)
}
#[tauri::command]
pub fn grade_submission(
    state: State<'_, AppState>,
    input: GradeSubmissionInput,
) -> Result<AssignmentSubmission, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let (assignment_id, student_id): (String, Option<String>) = conn
        .query_row(
            "SELECT assignment_id, student_id FROM assignment_submissions WHERE id = ?1",
            params![input.submission_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Submission not found".to_string())?;

    let mut points = input.points;
    let mut rubric_scores_json: Option<String> = None;
    if let Some(scores) = &input.rubric_scores {
        if let Some(rubric) = crate::commands::rubrics::rubric_for_assignment(&conn, &assignment_id)? {
            let mut total = 0.0;
            for score in scores {
                let criterion = rubric
                    .criteria
                    .iter()
                    .find(|c| c.id == score.criterion_id)
                    .ok_or_else(|| "Invalid rubric criterion".to_string())?;
                if score.points < 0.0 || score.points > criterion.max_points {
                    return Err(format!(
                        "Score for {} must be between 0 and {:.1}",
                        criterion.name, criterion.max_points
                    ));
                }
                total += score.points;
            }
            points = total;
            rubric_scores_json = Some(serde_json::to_string(scores).map_err(|e| e.to_string())?);
        }
    }

    conn.execute(
        "UPDATE assignment_submissions SET points = ?1, feedback = ?2, status = 'graded', graded_at = ?3 WHERE id = ?4",
        params![points, input.feedback, now, input.submission_id],
    )
    .map_err(|e| e.to_string())?;

    if let Some(sid) = student_id {
        conn.execute(
            "INSERT INTO grades (id, assignment_id, student_id, points, feedback, graded_at, rubric_scores_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) ON CONFLICT(assignment_id, student_id) DO UPDATE SET points = excluded.points, feedback = excluded.feedback, graded_at = excluded.graded_at, rubric_scores_json = excluded.rubric_scores_json",
            params![
                Uuid::new_v4().to_string(),
                assignment_id,
                sid,
                points,
                input.feedback,
                now,
                rubric_scores_json
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT sub.id, sub.assignment_id, a.title, sub.student_name, sub.student_id, sub.body, sub.file_name, sub.file_data, sub.points, sub.feedback, sub.status, sub.submitted_at, sub.graded_at FROM assignment_submissions sub JOIN assignments a ON a.id = sub.assignment_id WHERE sub.id = ?1",
        params![input.submission_id],
        map_submission_row,
    )
    .map_err(|e| e.to_string())
}
