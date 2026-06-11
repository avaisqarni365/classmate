use crate::models::{CreateScheduleSlotInput, ScheduleSlot};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_schedule(
    state: State<'_, AppState>,
    course_id: Option<String>,
) -> Result<Vec<ScheduleSlot>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sql = if course_id.is_some() {
        "SELECT s.id, s.course_id, c.title, c.code, s.day_of_week, s.start_time, s.end_time, s.room, s.title
         FROM schedule_slots s
         JOIN courses c ON c.id = s.course_id
         WHERE s.course_id = ?1
         ORDER BY s.day_of_week ASC, s.start_time ASC"
    } else {
        "SELECT s.id, s.course_id, c.title, c.code, s.day_of_week, s.start_time, s.end_time, s.room, s.title
         FROM schedule_slots s
         JOIN courses c ON c.id = s.course_id
         ORDER BY s.day_of_week ASC, s.start_time ASC, c.title ASC"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = if let Some(cid) = course_id {
        stmt.query_map(params![cid], map_slot_row)
    } else {
        stmt.query_map([], map_slot_row)
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(rows)
}

fn map_slot_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScheduleSlot> {
    Ok(ScheduleSlot {
        id: row.get(0)?,
        course_id: row.get(1)?,
        course_title: row.get(2)?,
        course_code: row.get(3)?,
        day_of_week: row.get(4)?,
        start_time: row.get(5)?,
        end_time: row.get(6)?,
        room: row.get(7)?,
        title: row.get(8)?,
    })
}

#[tauri::command]
pub fn create_schedule_slot(
    state: State<'_, AppState>,
    input: CreateScheduleSlotInput,
) -> Result<ScheduleSlot, String> {
    if input.day_of_week < 0 || input.day_of_week > 6 {
        return Err("day_of_week must be 0 (Mon) through 6 (Sun)".into());
    }
    if input.start_time.trim().is_empty() || input.end_time.trim().is_empty() {
        return Err("Start and end times are required".into());
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let (course_title, course_code): (String, String) = conn
        .query_row(
            "SELECT title, code FROM courses WHERE id = ?1",
            params![input.course_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Course not found".to_string())?;

    conn.execute(
        "INSERT INTO schedule_slots (id, course_id, day_of_week, start_time, end_time, room, title, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.course_id,
            input.day_of_week,
            input.start_time.trim(),
            input.end_time.trim(),
            input.room,
            input.title,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(ScheduleSlot {
        id,
        course_id: input.course_id,
        course_title,
        course_code,
        day_of_week: input.day_of_week,
        start_time: input.start_time.trim().to_string(),
        end_time: input.end_time.trim().to_string(),
        room: input.room,
        title: input.title,
    })
}

#[tauri::command]
pub fn delete_schedule_slot(state: State<'_, AppState>, slot_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM schedule_slots WHERE id = ?1",
        params![slot_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
