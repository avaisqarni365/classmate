use crate::models::User;
use crate::AppState;
use rusqlite::{params, Connection};
use tauri::State;

pub fn require_user(state: &State<'_, AppState>) -> Result<User, String> {
    state
        .session
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "Not authenticated".into())
}

pub fn get_active_school_id(state: &State<'_, AppState>) -> Option<String> {
    state
        .active_school_id
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

pub fn set_active_school_id(state: &State<'_, AppState>, school_id: Option<String>) {
    if let Ok(mut guard) = state.active_school_id.lock() {
        *guard = school_id;
    }
}

pub fn require_org_admin(user: &User) -> Result<(), String> {
    if user.role == "admin" {
        Ok(())
    } else {
        Err("Org admin access required".into())
    }
}

pub fn user_can_access_school(
    conn: &Connection,
    user_id: &str,
    school_id: &str,
) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM school_members WHERE user_id = ?1 AND school_id = ?2",
            params![user_id, school_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn require_school_access(
    conn: &Connection,
    user_id: &str,
    school_id: &str,
) -> Result<(), String> {
    if user_can_access_school(conn, user_id, school_id)? {
        Ok(())
    } else {
        Err("You do not have access to this school".into())
    }
}

pub fn resolve_active_school(conn: &Connection, user_id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT sm.school_id FROM school_members sm
         JOIN schools s ON s.id = sm.school_id
         WHERE sm.user_id = ?1
         ORDER BY s.name COLLATE NOCASE ASC
         LIMIT 1",
        params![user_id],
        |row| row.get(0),
    )
    .map_err(|_| "No school membership found for this user".to_string())
}

pub fn active_school_id_or_resolve(
    state: &State<'_, AppState>,
    conn: &Connection,
) -> Result<String, String> {
    if let Some(id) = get_active_school_id(state) {
        return Ok(id);
    }
    let user = require_user(state)?;
    let school_id = resolve_active_school(conn, &user.id)?;
    set_active_school_id(state, Some(school_id.clone()));
    Ok(school_id)
}

pub fn course_in_active_school(
    conn: &Connection,
    course_id: &str,
    school_id: &str,
) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM courses WHERE id = ?1 AND school_id = ?2",
            params![course_id, school_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn require_course_in_active_school(
    state: &State<'_, AppState>,
    conn: &Connection,
    course_id: &str,
) -> Result<String, String> {
    let school_id = active_school_id_or_resolve(state, conn)?;
    if course_in_active_school(conn, course_id, &school_id)? {
        Ok(school_id)
    } else {
        Err("Course not found in the active school".into())
    }
}
