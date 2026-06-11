use crate::models::{Announcement, CreateAnnouncementInput};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_announcements(
    state: State<'_, AppState>,
    course_id: Option<String>,
) -> Result<Vec<Announcement>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut sql = String::from(
        "SELECT id, course_id, title, body, author_id, created_at FROM announcements",
    );
    if course_id.is_some() {
        sql.push_str(" WHERE course_id = ?1 OR course_id IS NULL");
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 100");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = if let Some(ref cid) = course_id {
        stmt.query_map(params![cid], map_row)
    } else {
        stmt.query_map([], map_row)
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(rows)
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Announcement> {
    Ok(Announcement {
        id: row.get(0)?,
        course_id: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        author_id: row.get(4)?,
        created_at: row.get(5)?,
    })
}

#[tauri::command]
pub fn create_announcement(
    state: State<'_, AppState>,
    input: CreateAnnouncementInput,
) -> Result<Announcement, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let author_id = state
        .session
        .lock()
        .map_err(|e| e.to_string())?
        .as_ref()
        .map(|u| u.id.clone());

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO announcements (id, course_id, title, body, author_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            input.course_id,
            input.title.trim(),
            input.body.trim(),
            author_id,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Announcement {
        id,
        course_id: input.course_id,
        title: input.title.trim().to_string(),
        body: input.body.trim().to_string(),
        author_id,
        created_at: now,
    })
}

#[tauri::command]
pub fn delete_announcement(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM announcements WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
