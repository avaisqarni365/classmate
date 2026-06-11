pub mod analytics;
pub mod announcements;
pub mod auth;
pub mod backup;
pub mod cashbook;
pub mod email;
pub mod enrollments;
pub mod forums;
pub mod gradebook;
pub mod lti;
pub mod materials;
pub mod parent;
pub mod polls;
pub mod push;
pub mod quizzes;
pub mod rubrics;
pub mod schedule;
pub mod schools;
pub mod sessions;
pub mod student;
pub mod submissions;
pub mod sync;
pub mod tenancy;
pub mod video;
pub mod whatsapp;
pub mod whatsapp_api;

use crate::commands::sync::resolve_hub_join_url;
use crate::commands::tenancy::active_school_id_or_resolve;
use crate::models::{
    Assignment, Course, CreateCourseInput, CreateUserInput, DashboardStats, HubStatus,
    StartHubInput, User,
};
use crate::AppState;
use bcrypt::hash;
use chrono::Utc;
use rand::Rng;
use rusqlite::params;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

fn generate_pin() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1_000_000))
}

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;

    let user_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM school_members WHERE school_id = ?1",
            params![school_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let course_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM courses WHERE school_id = ?1",
            params![school_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let student_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM school_members sm
             JOIN users u ON u.id = sm.user_id
             WHERE sm.school_id = ?1 AND u.role = 'student'",
            params![school_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let assignment_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM assignments a
             JOIN courses c ON c.id = a.course_id
             WHERE c.school_id = ?1",
            params![school_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let active_sessions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM class_sessions cs
             JOIN courses c ON c.id = cs.course_id
             WHERE cs.status = 'live' AND c.school_id = ?1",
            params![school_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(DashboardStats {
        user_count,
        course_count,
        student_count,
        assignment_count,
        active_sessions,
    })
}

#[tauri::command]
pub fn list_users(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.email, u.name, u.role, u.phone, u.created_at
             FROM users u
             JOIN school_members sm ON sm.user_id = u.id
             WHERE sm.school_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map(params![school_id], |row| {
            Ok(User {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                role: row.get(3)?,
                phone: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(users)
}

#[tauri::command]
pub fn create_user(state: State<'_, AppState>, input: CreateUserInput) -> Result<User, String> {
    if !matches!(input.role.as_str(), "admin" | "teacher" | "student" | "parent") {
        return Err("Invalid role".into());
    }
    if input.password.len() < 6 {
        return Err("Password must be at least 6 characters".into());
    }

    let password_hash = hash(input.password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    conn.execute(
        "INSERT INTO users (id, email, name, role, password_hash, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, input.email, input.name, input.role, password_hash, now, now],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO school_members (id, school_id, user_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![Uuid::new_v4().to_string(), school_id, id, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(User {
        id,
        email: input.email,
        name: input.name,
        role: input.role,
        phone: None,
        created_at: now,
    })
}

#[tauri::command]
pub fn list_courses(state: State<'_, AppState>) -> Result<Vec<Course>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.title, c.code, c.description, c.teacher_id, u.name,
                    c.term, COUNT(e.id) AS student_count, c.created_at
             FROM courses c
             LEFT JOIN users u ON u.id = c.teacher_id
             LEFT JOIN enrollments e ON e.course_id = c.id AND e.status = 'active'
             WHERE c.school_id = ?1
             GROUP BY c.id
             ORDER BY c.title COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let courses = stmt
        .query_map(params![school_id], |row| {
            Ok(Course {
                id: row.get(0)?,
                title: row.get(1)?,
                code: row.get(2)?,
                description: row.get(3)?,
                teacher_id: row.get(4)?,
                teacher_name: row.get(5)?,
                term: row.get(6)?,
                student_count: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(courses)
}

#[tauri::command]
pub fn create_course(
    state: State<'_, AppState>,
    input: CreateCourseInput,
) -> Result<Course, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    conn.execute(
        "INSERT INTO courses (id, title, code, description, teacher_id, term, school_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            input.title,
            input.code,
            input.description,
            input.teacher_id,
            input.term,
            school_id,
            now,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    let teacher_name: Option<String> = if let Some(ref tid) = input.teacher_id {
        conn.query_row(
            "SELECT name FROM users WHERE id = ?1",
            params![tid],
            |row| row.get(0),
        )
        .ok()
    } else {
        None
    };

    Ok(Course {
        id,
        title: input.title,
        code: input.code,
        description: input.description,
        teacher_id: input.teacher_id,
        teacher_name,
        term: input.term,
        student_count: 0,
        created_at: now,
    })
}

#[tauri::command]
pub fn list_assignments(
    state: State<'_, AppState>,
    course_id: String,
) -> Result<Vec<Assignment>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, course_id, title, description, due_at, max_points, created_at
             FROM assignments WHERE course_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let assignments = stmt
        .query_map(params![course_id], |row| {
            Ok(Assignment {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                due_at: row.get(4)?,
                max_points: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(assignments)
}

#[tauri::command]
pub fn get_hub_status(state: State<'_, AppState>) -> Result<HubStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let hub = state.hub.lock().map_err(|e| e.to_string())?;
    let mut status = hub.status();
    status.join_url = resolve_hub_join_url(&conn, status.join_url);
    Ok(status)
}

#[tauri::command]
pub fn start_class_hub(
    state: State<'_, AppState>,
    app: AppHandle,
    input: StartHubInput,
) -> Result<HubStatus, String> {
    let course_title: String;
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        course_title = conn
            .query_row(
                "SELECT title FROM courses WHERE id = ?1",
                params![input.course_id],
                |row| row.get(0),
            )
            .map_err(|_| "Course not found".to_string())?;
    }

    let session_title = input
        .title
        .unwrap_or_else(|| format!("Live: {}", course_title));
    let session_id = Uuid::new_v4().to_string();
    let pin = generate_pin();
    let now = Utc::now().to_rfc3339();

    let mut video_url: Option<String> = None;
    if input.enable_video.unwrap_or(true) {
        let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        video_url = backup::resolve_hub_video_url(&conn, &app_data, &state.video);
    }

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE class_sessions SET status = 'ended', ended_at = ?1 WHERE status = 'live'",
            params![now],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO class_sessions (id, course_id, title, pin, status, started_at, created_at)
             VALUES (?1, ?2, ?3, ?4, 'live', ?5, ?6)",
            params![
                session_id,
                input.course_id,
                session_title,
                pin,
                now,
                now
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    let db = state.db.clone();
    let mut hub = state.hub.lock().map_err(|e| e.to_string())?;
    hub.start(
        db,
        session_id.clone(),
        input.course_id.clone(),
        course_title,
        pin.clone(),
        video_url,
    )
    .map_err(|e| e.to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut status = hub.status();
    status.join_url = resolve_hub_join_url(&conn, status.join_url);
    Ok(status)
}

#[tauri::command]
pub fn stop_class_hub(state: State<'_, AppState>) -> Result<HubStatus, String> {
    let now = Utc::now().to_rfc3339();
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE class_sessions SET status = 'ended', ended_at = ?1 WHERE status = 'live'",
            params![now],
        )
        .map_err(|e| e.to_string())?;
    }

    state.video.stop();
    let mut hub = state.hub.lock().map_err(|e| e.to_string())?;
    hub.stop();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut status = hub.status();
    status.join_url = resolve_hub_join_url(&conn, status.join_url);
    Ok(status)
}

#[tauri::command]
pub fn get_attendance_count(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM attendance WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count)
}
