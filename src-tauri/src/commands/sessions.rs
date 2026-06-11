use crate::models::ClassSessionRecord;
use crate::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_class_sessions(
    state: State<'_, AppState>,
    course_id: Option<String>,
) -> Result<Vec<ClassSessionRecord>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sql = if course_id.is_some() {
        "SELECT cs.id, cs.course_id, c.title, cs.title, cs.status, cs.pin, cs.started_at, cs.ended_at,
                COUNT(att.id) AS attendance_count
         FROM class_sessions cs
         JOIN courses c ON c.id = cs.course_id
         LEFT JOIN attendance att ON att.session_id = cs.id
         WHERE cs.course_id = ?1
         GROUP BY cs.id
         ORDER BY cs.started_at DESC"
    } else {
        "SELECT cs.id, cs.course_id, c.title, cs.title, cs.status, cs.pin, cs.started_at, cs.ended_at,
                COUNT(att.id) AS attendance_count
         FROM class_sessions cs
         JOIN courses c ON c.id = cs.course_id
         LEFT JOIN attendance att ON att.session_id = cs.id
         GROUP BY cs.id
         ORDER BY cs.started_at DESC
         LIMIT 100"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = if let Some(cid) = course_id {
        stmt.query_map(params![cid], map_session)
    } else {
        stmt.query_map([], map_session)
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn map_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClassSessionRecord> {
    Ok(ClassSessionRecord {
        id: row.get(0)?,
        course_id: row.get(1)?,
        course_title: row.get(2)?,
        title: row.get(3)?,
        status: row.get(4)?,
        pin: row.get(5)?,
        started_at: row.get(6)?,
        ended_at: row.get(7)?,
        attendance_count: row.get(8)?,
    })
}
