use crate::commands::tenancy::{
    active_school_id_or_resolve, require_org_admin, require_school_access, require_user,
    set_active_school_id, user_can_access_school,
};
use crate::models::{
    AddSchoolMemberInput, CreateSchoolInput, School, SchoolMember, TenancyContext,
    UpdateSchoolInput,
};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

fn map_school(row: &rusqlite::Row<'_>) -> rusqlite::Result<School> {
    Ok(School {
        id: row.get(0)?,
        name: row.get(1)?,
        code: row.get(2)?,
        created_at: row.get(3)?,
    })
}

fn school_summaries_for_user(
    conn: &rusqlite::Connection,
    user_id: &str,
) -> Result<Vec<crate::models::SchoolSummary>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.name, s.code,
                    (SELECT COUNT(*) FROM school_members sm2 WHERE sm2.school_id = s.id) AS member_count,
                    (SELECT COUNT(*) FROM courses c WHERE c.school_id = s.id) AS course_count
             FROM schools s
             JOIN school_members sm ON sm.school_id = s.id
             WHERE sm.user_id = ?1
             ORDER BY s.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let schools = stmt
        .query_map(params![user_id], |row| {
            Ok(crate::models::SchoolSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                member_count: row.get(3)?,
                course_count: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(schools)
}

fn build_tenancy_context(
    conn: &rusqlite::Connection,
    user: &crate::models::User,
    active_school_id: &str,
) -> Result<TenancyContext, String> {
    let active_school_name: String = conn
        .query_row(
            "SELECT name FROM schools WHERE id = ?1",
            params![active_school_id],
            |row| row.get(0),
        )
        .map_err(|_| "Active school not found".to_string())?;

    Ok(TenancyContext {
        active_school_id: active_school_id.to_string(),
        active_school_name,
        schools: school_summaries_for_user(conn, &user.id)?,
        is_org_admin: user.role == "admin",
    })
}

#[tauri::command]
pub fn get_tenancy_context(state: State<'_, AppState>) -> Result<TenancyContext, String> {
    let user = require_user(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    build_tenancy_context(&conn, &user, &school_id)
}

#[tauri::command]
pub fn set_active_school(
    state: State<'_, AppState>,
    school_id: String,
) -> Result<TenancyContext, String> {
    let user = require_user(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    require_school_access(&conn, &user.id, &school_id)?;
    set_active_school_id(&state, Some(school_id.clone()));
    build_tenancy_context(&conn, &user, &school_id)
}

#[tauri::command]
pub fn create_school(
    state: State<'_, AppState>,
    input: CreateSchoolInput,
) -> Result<School, String> {
    let user = require_user(&state)?;
    require_org_admin(&user)?;

    let name = input.name.trim();
    let code = input.code.trim().to_uppercase();
    if name.is_empty() || code.is_empty() {
        return Err("Name and code are required".into());
    }

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO schools (id, name, code, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, code, now],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO school_members (id, school_id, user_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![Uuid::new_v4().to_string(), id, user.id, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(School {
        id,
        name: name.to_string(),
        code,
        created_at: now,
    })
}

#[tauri::command]
pub fn update_school(
    state: State<'_, AppState>,
    input: UpdateSchoolInput,
) -> Result<School, String> {
    let user = require_user(&state)?;
    require_org_admin(&user)?;

    let name = input.name.trim();
    let code = input.code.trim().to_uppercase();
    if name.is_empty() || code.is_empty() {
        return Err("Name and code are required".into());
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    require_school_access(&conn, &user.id, &input.id)?;

    conn.execute(
        "UPDATE schools SET name = ?1, code = ?2 WHERE id = ?3",
        params![name, code, input.id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, code, created_at FROM schools WHERE id = ?1",
        params![input.id],
        map_school,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_school_members(
    state: State<'_, AppState>,
    school_id: String,
) -> Result<Vec<SchoolMember>, String> {
    let user = require_user(&state)?;
    require_org_admin(&user)?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    require_school_access(&conn, &user.id, &school_id)?;

    let mut stmt = conn
        .prepare(
            "SELECT sm.id, sm.school_id, sm.user_id, u.name, u.email, u.role, sm.created_at
             FROM school_members sm
             JOIN users u ON u.id = sm.user_id
             WHERE sm.school_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map(params![school_id], |row| {
            Ok(SchoolMember {
                id: row.get(0)?,
                school_id: row.get(1)?,
                user_id: row.get(2)?,
                user_name: row.get(3)?,
                user_email: row.get(4)?,
                user_role: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(members)
}

#[tauri::command]
pub fn add_school_member(
    state: State<'_, AppState>,
    input: AddSchoolMemberInput,
) -> Result<SchoolMember, String> {
    let user = require_user(&state)?;
    require_org_admin(&user)?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    require_school_access(&conn, &user.id, &input.school_id)?;

    let target: (String, String, String, String) = conn
        .query_row(
            "SELECT id, name, email, role FROM users WHERE email = ?1",
            params![input.user_email.trim()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| "User not found".to_string())?;

    let now = Utc::now().to_rfc3339();
    let membership_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO school_members (id, school_id, user_id, created_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(school_id, user_id) DO NOTHING",
        params![membership_id, input.school_id, target.0, now],
    )
    .map_err(|e| e.to_string())?;

    let id: String = conn
        .query_row(
            "SELECT id FROM school_members WHERE school_id = ?1 AND user_id = ?2",
            params![input.school_id, target.0],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(SchoolMember {
        id,
        school_id: input.school_id,
        user_id: target.0,
        user_name: target.1,
        user_email: target.2,
        user_role: target.3,
        created_at: now,
    })
}

#[tauri::command]
pub fn remove_school_member(
    state: State<'_, AppState>,
    membership_id: String,
) -> Result<(), String> {
    let user = require_user(&state)?;
    require_org_admin(&user)?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (school_id, target_user_id): (String, String) = conn
        .query_row(
            "SELECT school_id, user_id FROM school_members WHERE id = ?1",
            params![membership_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Membership not found".to_string())?;

    require_school_access(&conn, &user.id, &school_id)?;

    let member_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM school_members WHERE school_id = ?1",
            params![school_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if member_count <= 1 {
        return Err("Cannot remove the last member from a school".into());
    }

    if target_user_id == user.id {
        return Err("Cannot remove yourself from the active school".into());
    }

    conn.execute(
        "DELETE FROM school_members WHERE id = ?1",
        params![membership_id],
    )
    .map_err(|e| e.to_string())?;

    if user_can_access_school(&conn, &target_user_id, &school_id)? {
        return Ok(());
    }

    Ok(())
}
