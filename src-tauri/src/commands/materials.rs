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
    list_materials_for_course(&conn, &course_id)
}

pub fn list_materials_for_course(
    conn: &rusqlite::Connection,
    course_id: &str,
) -> Result<Vec<CourseMaterial>, String> {
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

pub fn lectures_for_course(
    conn: &rusqlite::Connection,
    course_id: &str,
) -> Result<Vec<crate::models::MaterialWithAiLab>, String> {
    lectures_for_course_student(conn, course_id, None)
}

pub fn lectures_for_course_student(
    conn: &rusqlite::Connection,
    course_id: &str,
    student_id: Option<&str>,
) -> Result<Vec<crate::models::MaterialWithAiLab>, String> {
    let (course_code, course_title): (String, String) = conn
        .query_row(
            "SELECT code, title FROM courses WHERE id = ?1",
            params![course_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Course not found".to_string())?;
    let materials = list_materials_for_course(conn, course_id)?;
    Ok(materials
        .into_iter()
        .map(|material| {
            let subjects = textbook_subjects(&material);
            let ai_lab = crate::commands::ai_lab::build_material_ai_lab(
                conn,
                &course_code,
                &course_title,
                &material.title,
                subjects.as_deref(),
                material_textbook_slug(&material),
            );
            let lab_completed = student_id
                .and_then(|sid| {
                    crate::commands::notes::lab_completion_for_student(conn, &material.id, sid)
                })
                .is_some();
            crate::models::MaterialWithAiLab {
                material,
                ai_lab,
                lab_completed,
            }
        })
        .collect())
}

fn material_textbook_slug(material: &CourseMaterial) -> Option<String> {
    if material.kind != "textbook" {
        return None;
    }
    let parsed: serde_json::Value = serde_json::from_str(&material.content).ok()?;
    parsed
        .get("book_slug")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn textbook_subjects(material: &CourseMaterial) -> Option<Vec<String>> {
    if material.kind != "textbook" {
        return None;
    }
    let parsed: serde_json::Value = serde_json::from_str(&material.content).ok()?;
    parsed
        .get("subjects")
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

#[tauri::command]
pub fn create_material(
    state: State<'_, AppState>,
    input: CreateMaterialInput,
) -> Result<CourseMaterial, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    create_material_work(&conn, &input)
}

pub fn create_material_work(
    conn: &rusqlite::Connection,
    input: &CreateMaterialInput,
) -> Result<CourseMaterial, String> {
    if input.title.trim().is_empty() || input.content.trim().is_empty() {
        return Err("Title and content are required".into());
    }

    let kind = match input.kind.as_str() {
        "note" | "link" | "file" | "textbook" | "speak_note" | "handwriting" => input.kind.as_str(),
        _ => "note",
    };

    if matches!(kind, "textbook" | "speak_note" | "handwriting") {
        serde_json::from_str::<serde_json::Value>(input.content.trim())
            .map_err(|_| "Structured note content must be valid JSON".to_string())?;
    }

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

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
        course_id: input.course_id.clone(),
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

#[tauri::command]
pub fn list_course_lectures(
    state: State<'_, AppState>,
    course_id: String,
) -> Result<Vec<crate::models::MaterialWithAiLab>, String> {
    let student_id = {
        let session = state.session.lock().map_err(|e| e.to_string())?;
        session
            .as_ref()
            .filter(|u| u.role == "student")
            .map(|u| u.id.clone())
    };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    lectures_for_course_student(
        &conn,
        &course_id,
        student_id.as_deref(),
    )
}
