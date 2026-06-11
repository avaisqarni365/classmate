use crate::commands::tenancy::{resolve_active_school, set_active_school_id};
use crate::models::{LoginInput, User};
use crate::AppState;
use bcrypt::verify;
use rusqlite::params;
use tauri::State;

fn row_to_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        email: row.get(1)?,
        name: row.get(2)?,
        role: row.get(3)?,
        phone: row.get(4)?,
        created_at: row.get(5)?,
    })
}

#[tauri::command]
pub fn login(state: State<'_, AppState>, input: LoginInput) -> Result<User, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let row = conn.query_row(
        "SELECT id, email, name, role, phone, created_at, password_hash FROM users WHERE email = ?1",
        params![input.email.trim()],
        |row| {
            Ok((
                User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    name: row.get(2)?,
                    role: row.get(3)?,
                    phone: row.get(4)?,
                    created_at: row.get(5)?,
                },
                row.get::<_, String>(6)?,
            ))
        },
    );

    let (user, hash) = row.map_err(|_| "Invalid email or password".to_string())?;
    let valid = verify(input.password, &hash).map_err(|e| e.to_string())?;
    if !valid {
        return Err("Invalid email or password".into());
    }

    *state.session.lock().map_err(|e| e.to_string())? = Some(user.clone());
    let school_id = resolve_active_school(&conn, &user.id)?;
    set_active_school_id(&state, Some(school_id));
    Ok(user)
}

#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> Result<(), String> {
    *state.session.lock().map_err(|e| e.to_string())? = None;
    set_active_school_id(&state, None);
    Ok(())
}

#[tauri::command]
pub fn get_session(state: State<'_, AppState>) -> Result<Option<User>, String> {
    Ok(state.session.lock().map_err(|e| e.to_string())?.clone())
}

pub fn require_user(state: &State<'_, AppState>) -> Result<User, String> {
    state
        .session
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "Not authenticated".into())
}

#[allow(dead_code)]
pub fn fetch_user_by_id(conn: &rusqlite::Connection, id: &str) -> Result<User, String> {
    conn.query_row(
        "SELECT id, email, name, role, phone, created_at FROM users WHERE id = ?1",
        params![id],
        row_to_user,
    )
    .map_err(|e| e.to_string())
}
