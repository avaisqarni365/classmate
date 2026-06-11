use crate::models::{CreateLtiToolInput, LtiTool};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_lti_tools(state: State<'_, AppState>) -> Result<Vec<LtiTool>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, launch_url, consumer_key, created_at FROM lti_tools ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(LtiTool {
                id: row.get(0)?,
                name: row.get(1)?,
                launch_url: row.get(2)?,
                consumer_key: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn create_lti_tool(
    state: State<'_, AppState>,
    input: CreateLtiToolInput,
) -> Result<LtiTool, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO lti_tools (id, name, launch_url, consumer_key, shared_secret, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            input.name.trim(),
            input.launch_url.trim(),
            input.consumer_key.trim(),
            input.shared_secret.trim(),
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(LtiTool {
        id,
        name: input.name.trim().to_string(),
        launch_url: input.launch_url.trim().to_string(),
        consumer_key: input.consumer_key.trim().to_string(),
        created_at: now,
    })
}

#[tauri::command]
pub fn delete_lti_tool(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM lti_tools WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_lti_launch_url(
    state: State<'_, AppState>,
    tool_id: String,
    course_id: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (launch_url, consumer_key): (String, String) = conn
        .query_row(
            "SELECT launch_url, consumer_key FROM lti_tools WHERE id = ?1",
            params![tool_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "LTI tool not found".to_string())?;

    let course_title: String = conn
        .query_row(
            "SELECT title FROM courses WHERE id = ?1",
            params![course_id],
            |row| row.get(0),
        )
        .map_err(|_| "Course not found".to_string())?;

    Ok(format!(
        "{launch_url}?lti_message_type=basic-lti-launch-request&resource_link_id={course_id}&context_title={}&oauth_consumer_key={consumer_key}",
        urlencoding(&course_title)
    ))
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '&' => "%26".to_string(),
            _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
