use crate::models::{CourseMaterial, CreateMaterialInput};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_materials(
    state: State<'_, AppState>,
    course_id: String,
) -> Result<Vec<CourseMaterial>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, course_id, title, kind, content, created_at
             FROM course_materials WHERE course_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let materials = stmt
        .query_map(params![course_id], |row| {
            Ok(CourseMaterial {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                kind: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(materials)
}

#[tauri::command]
pub fn create_material(
    state: State<'_, AppState>,
    input: CreateMaterialInput,
) -> Result<CourseMaterial, String> {
    if input.title.trim().is_empty() || input.content.trim().is_empty() {
        return Err("Title and content are required".into());
    }

    let kind = match input.kind.as_str() {
        "note" | "link" | "file" => input.kind.as_str(),
        _ => "note",
    };

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO course_materials (id, course_id, title, kind, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            input.course_id,
            input.title.trim(),
            kind,
            input.content.trim(),
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(CourseMaterial {
        id,
        course_id: input.course_id,
        title: input.title.trim().to_string(),
        kind: kind.to_string(),
        content: input.content.trim().to_string(),
        created_at: now,
    })
}

#[tauri::command]
pub fn delete_material(state: State<'_, AppState>, material_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM course_materials WHERE id = ?1",
        params![material_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
