use crate::commands::tenancy::active_school_id_or_resolve;
use crate::models::{EnrollStudentInput, Enrollment};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_enrollments(
    state: State<'_, AppState>,
    course_id: String,
) -> Result<Vec<Enrollment>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.course_id, e.student_id, u.name, u.email, e.status, e.enrolled_at
             FROM enrollments e
             JOIN users u ON u.id = e.student_id
             WHERE e.course_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![course_id], |row| {
            Ok(Enrollment {
                id: row.get(0)?,
                course_id: row.get(1)?,
                student_id: row.get(2)?,
                student_name: row.get(3)?,
                student_email: row.get(4)?,
                status: row.get(5)?,
                enrolled_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn enroll_student(
    state: State<'_, AppState>,
    input: EnrollStudentInput,
) -> Result<Enrollment, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let role: String = conn
        .query_row(
            "SELECT role FROM users WHERE id = ?1",
            params![input.student_id],
            |row| row.get(0),
        )
        .map_err(|_| "Student not found".to_string())?;

    if role != "student" {
        return Err("Only students can be enrolled".into());
    }

    conn.execute(
        "INSERT INTO enrollments (id, course_id, student_id, status, enrolled_at)
         VALUES (?1, ?2, ?3, 'active', ?4)
         ON CONFLICT(course_id, student_id) DO UPDATE SET status = 'active'",
        params![id, input.course_id, input.student_id, now],
    )
    .map_err(|e| e.to_string())?;

    let enrollment_id: String = conn
        .query_row(
            "SELECT id FROM enrollments WHERE course_id = ?1 AND student_id = ?2",
            params![input.course_id, input.student_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let (student_name, student_email): (String, String) = conn
        .query_row(
            "SELECT name, email FROM users WHERE id = ?1",
            params![input.student_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(Enrollment {
        id: enrollment_id,
        course_id: input.course_id,
        student_id: input.student_id,
        student_name,
        student_email,
        status: "active".into(),
        enrolled_at: now,
    })
}

#[tauri::command]
pub fn unenroll_student(state: State<'_, AppState>, enrollment_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE enrollments SET status = 'inactive' WHERE id = ?1",
        params![enrollment_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_students(state: State<'_, AppState>) -> Result<Vec<crate::models::User>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.email, u.name, u.role, u.phone, u.created_at
             FROM users u
             JOIN school_members sm ON sm.user_id = u.id
             WHERE sm.school_id = ?1 AND u.role = 'student'
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map(params![school_id], |row| {
            Ok(crate::models::User {
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
