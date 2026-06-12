use crate::commands::materials::create_material_work;
use crate::commands::sync::read_public_base_url;
use crate::models::{
    CaptureSession, CourseMaterial, CreateCaptureSessionInput, CreateMaterialInput,
    MaterialLabCompletion, UpdateCaptureInkInput,
};
use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

#[tauri::command]
pub fn create_capture_session(
    state: tauri::State<'_, crate::AppState>,
    input: CreateCaptureSessionInput,
) -> Result<CaptureSession, String> {
    let user = {
        let session = state.session.lock().map_err(|e| e.to_string())?;
        session
            .clone()
            .ok_or_else(|| "Not logged in".to_string())?
    };
    if user.role != "teacher" && user.role != "admin" {
        return Err("Teacher access required".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    create_capture_session_work(
        &conn,
        &input.course_id,
        &user.id,
        &input.title,
    )
}

#[tauri::command]
pub fn get_capture_session(
    state: tauri::State<'_, crate::AppState>,
    session_id: String,
) -> Result<CaptureSession, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    load_capture_session(&conn, &session_id)
}

#[tauri::command]
pub fn update_capture_ink(
    state: tauri::State<'_, crate::AppState>,
    input: UpdateCaptureInkInput,
) -> Result<CaptureSession, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    update_capture_ink_work(&conn, &input)
}

#[tauri::command]
pub fn attach_capture_session(
    state: tauri::State<'_, crate::AppState>,
    session_id: String,
    title: Option<String>,
) -> Result<CourseMaterial, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    attach_capture_session_work(&conn, &session_id, title)
}

#[tauri::command]
pub fn get_capture_pad_url(
    state: tauri::State<'_, crate::AppState>,
    session_id: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    capture_pad_url(&conn, &session_id)
}

#[tauri::command]
pub fn mark_material_lab_complete(
    state: tauri::State<'_, crate::AppState>,
    material_id: String,
) -> Result<MaterialLabCompletion, String> {
    let user = {
        let session = state.session.lock().map_err(|e| e.to_string())?;
        session
            .clone()
            .ok_or_else(|| "Not logged in".to_string())?
    };
    if user.role != "student" {
        return Err("Student access only".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO material_lab_completions (id, material_id, student_id, completed_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(material_id, student_id) DO UPDATE SET completed_at = excluded.completed_at",
        params![id, material_id, user.id, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(MaterialLabCompletion {
        material_id,
        student_id: user.id,
        completed_at: now,
    })
}

pub fn create_capture_session_work(
    conn: &rusqlite::Connection,
    course_id: &str,
    created_by: &str,
    title: &str,
) -> Result<CaptureSession, String> {
    if title.trim().is_empty() {
        return Err("Title is required".into());
    }
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();
    let expires_at = (now + chrono::Duration::hours(2)).to_rfc3339();
    let now_str = now.to_rfc3339();
    conn.execute(
        "INSERT INTO note_capture_sessions
         (id, course_id, created_by, title, ink_json, preview_data_url, status, material_id, expires_at, updated_at, created_at)
         VALUES (?1, ?2, ?3, ?4, '[]', NULL, 'open', NULL, ?5, ?6, ?7)",
        params![id, course_id, created_by, title.trim(), expires_at, now_str, now_str],
    )
    .map_err(|e| e.to_string())?;
    load_capture_session(conn, &id)
}

pub fn load_capture_session(
    conn: &rusqlite::Connection,
    session_id: &str,
) -> Result<CaptureSession, String> {
    let session = conn
        .query_row(
            "SELECT id, course_id, created_by, title, ink_json, preview_data_url, status, material_id, expires_at, updated_at, created_at
             FROM note_capture_sessions WHERE id = ?1",
            params![session_id],
            |row| {
                Ok(CaptureSession {
                    id: row.get(0)?,
                    course_id: row.get(1)?,
                    created_by: row.get(2)?,
                    title: row.get(3)?,
                    ink_json: row.get(4)?,
                    preview_data_url: row.get(5)?,
                    status: row.get(6)?,
                    material_id: row.get(7)?,
                    expires_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    created_at: row.get(10)?,
                    pad_url: None,
                })
            },
        )
        .map_err(|_| "Capture session not found".to_string())?;
    let mut session = session;
    session.pad_url = Some(capture_pad_url(conn, session_id)?);
    Ok(session)
}

pub fn update_capture_ink_work(
    conn: &rusqlite::Connection,
    input: &UpdateCaptureInkInput,
) -> Result<CaptureSession, String> {
    let session = load_capture_session(conn, &input.session_id)?;
    if session.status != "open" {
        return Err("Capture session is no longer open".into());
    }
    if Utc::now()
        > session
            .expires_at
            .parse::<chrono::DateTime<Utc>>()
            .map_err(|e| e.to_string())?
    {
        conn.execute(
            "UPDATE note_capture_sessions SET status = 'expired' WHERE id = ?1",
            params![input.session_id],
        )
        .ok();
        return Err("Capture session expired".into());
    }
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE note_capture_sessions SET ink_json = ?1, preview_data_url = ?2, updated_at = ?3 WHERE id = ?4",
        params![
            input.ink_json,
            input.preview_data_url,
            now,
            input.session_id
        ],
    )
    .map_err(|e| e.to_string())?;
    load_capture_session(conn, &input.session_id)
}

pub fn attach_capture_session_work(
    conn: &rusqlite::Connection,
    session_id: &str,
    title: Option<String>,
) -> Result<CourseMaterial, String> {
    let session = load_capture_session(conn, session_id)?;
    if session.status != "open" {
        return Err("Capture session is not open".into());
    }
    let material_title = title
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| session.title.clone());
    let content = serde_json::json!({
        "ink_json": session.ink_json,
        "preview_data_url": session.preview_data_url,
        "session_id": session.id,
    })
    .to_string();
    let material = create_material_work(
        conn,
        &CreateMaterialInput {
            course_id: session.course_id.clone(),
            title: material_title,
            kind: "handwriting".into(),
            content,
        },
    )?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE note_capture_sessions SET status = 'attached', material_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![material.id, now, session_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(material)
}

pub fn capture_pad_url(conn: &rusqlite::Connection, session_id: &str) -> Result<String, String> {
    let base = read_public_base_url(conn).unwrap_or_else(|| "http://127.0.0.1:8766".to_string());
    Ok(format!(
        "{}/notes/pad?session={}",
        base.trim_end_matches('/'),
        session_id
    ))
}

pub fn lab_completion_for_student(
    conn: &rusqlite::Connection,
    material_id: &str,
    student_id: &str,
) -> Option<MaterialLabCompletion> {
    conn.query_row(
        "SELECT material_id, student_id, completed_at FROM material_lab_completions
         WHERE material_id = ?1 AND student_id = ?2",
        params![material_id, student_id],
        |row| {
            Ok(MaterialLabCompletion {
                material_id: row.get(0)?,
                student_id: row.get(1)?,
                completed_at: row.get(2)?,
            })
        },
    )
    .ok()
}
