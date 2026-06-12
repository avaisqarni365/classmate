use crate::commands::rubrics::rubric_scores_display;
use crate::commands::tenancy::active_school_id_or_resolve;
use crate::models::{
    AssignmentSubmission, CourseMaterial, StudentAssignmentGrade, StudentCourseDetail,
    StudentCourseSummary, StudentDashboard, SubmitMyAssignmentInput, User,
};
use crate::AppState;
use rusqlite::params;
use tauri::State;

fn require_student(state: &State<'_, AppState>) -> Result<User, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    let user = session.as_ref().ok_or("Not logged in")?.clone();
    if user.role != "student" {
        return Err("Student access only".into());
    }
    Ok(user)
}

fn map_assignment_grade(
    conn: &rusqlite::Connection,
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StudentAssignmentGrade> {
    let assignment_id: String = row.get(0)?;
    let rubric_scores_json: Option<String> = row.get(8).ok();
    let rubric_scores = rubric_scores_display(
        conn,
        &assignment_id,
        rubric_scores_json.as_deref(),
    )
    .ok()
    .flatten();
    Ok(StudentAssignmentGrade {
        assignment_id,
        course_id: row.get(1)?,
        course_title: row.get(2)?,
        title: row.get(3)?,
        due_at: row.get(4)?,
        max_points: row.get(5)?,
        points: row.get(6)?,
        feedback: row.get(7)?,
        rubric_scores,
    })
}

fn course_summary(
    conn: &rusqlite::Connection,
    course_id: &str,
    student_id: &str,
) -> Result<StudentCourseSummary, String> {
    let (title, code, teacher_name): (String, String, Option<String>) = conn
        .query_row(
            "SELECT c.title, c.code, u.name FROM courses c
             LEFT JOIN users u ON u.id = c.teacher_id
             WHERE c.id = ?1",
            params![course_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Course not found".to_string())?;

    let assignment_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM assignments WHERE course_id = ?1",
            params![course_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let graded_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM grades g
             JOIN assignments a ON a.id = g.assignment_id
             WHERE a.course_id = ?1 AND g.student_id = ?2 AND g.points IS NOT NULL",
            params![course_id, student_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let average_percent: Option<f64> = conn
        .query_row(
            "SELECT AVG((g.points / a.max_points) * 100)
             FROM grades g
             JOIN assignments a ON a.id = g.assignment_id
             WHERE a.course_id = ?1 AND g.student_id = ?2 AND g.points IS NOT NULL AND a.max_points > 0",
            params![course_id, student_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    Ok(StudentCourseSummary {
        course_id: course_id.to_string(),
        title,
        code,
        teacher_name,
        average_percent,
        assignment_count,
        graded_count,
    })
}

#[tauri::command]
pub fn get_student_dashboard(state: State<'_, AppState>) -> Result<StudentDashboard, String> {
    let user = require_student(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    build_student_dashboard(&conn, &user.id, &school_id)
}

pub fn build_student_dashboard(
    conn: &rusqlite::Connection,
    student_id: &str,
    school_id: &str,
) -> Result<StudentDashboard, String> {
    let mut course_stmt = conn
        .prepare(
            "SELECT c.id FROM courses c
             JOIN enrollments e ON e.course_id = c.id
             WHERE e.student_id = ?1 AND e.status = 'active' AND c.school_id = ?2
             ORDER BY c.title ASC",
        )
        .map_err(|e| e.to_string())?;

    let course_ids: Vec<String> = course_stmt
        .query_map(params![student_id, school_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut courses = Vec::new();
    for cid in &course_ids {
        courses.push(course_summary(conn, cid, student_id)?);
    }

    let mut upcoming_stmt = conn
        .prepare(
            "SELECT a.id, a.course_id, c.title, a.title, a.due_at, a.max_points, g.points, g.feedback, g.rubric_scores_json
             FROM assignments a
             JOIN courses c ON c.id = a.course_id
             JOIN enrollments e ON e.course_id = c.id AND e.student_id = ?1 AND e.status = 'active'
             LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = ?1
             WHERE c.school_id = ?2
             ORDER BY CASE WHEN g.points IS NULL THEN 0 ELSE 1 END, a.due_at ASC
             LIMIT 10",
        )
        .map_err(|e| e.to_string())?;

    let upcoming = upcoming_stmt
        .query_map(params![student_id, school_id], |row| map_assignment_grade(conn, row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(StudentDashboard { courses, upcoming })
}

#[tauri::command]
pub fn get_my_course(
    state: State<'_, AppState>,
    course_id: String,
) -> Result<StudentCourseDetail, String> {
    let user = require_student(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;

    let enrolled: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM enrollments e
             JOIN courses c ON c.id = e.course_id
             WHERE e.course_id = ?1 AND e.student_id = ?2 AND e.status = 'active' AND c.school_id = ?3",
            params![course_id, user.id, school_id],
            |row| row.get(0),
        )
        .map(|c: i64| c > 0)
        .unwrap_or(false);
    if !enrolled {
        return Err("You are not enrolled in this course".into());
    }

    let course = course_summary(&conn, &course_id, &user.id)?;

    let mut assignment_stmt = conn
        .prepare(
            "SELECT a.id, a.course_id, c.title, a.title, a.due_at, a.max_points, g.points, g.feedback, g.rubric_scores_json
             FROM assignments a
             JOIN courses c ON c.id = a.course_id
             LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = ?2
             WHERE a.course_id = ?1
             ORDER BY a.created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let assignments = assignment_stmt
        .query_map(params![course_id, user.id], |row| map_assignment_grade(&conn, row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut material_stmt = conn
        .prepare(
            "SELECT id, course_id, title, kind, content, created_at FROM course_materials
             WHERE course_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let materials = material_stmt
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

    Ok(StudentCourseDetail {
        course,
        assignments,
        materials,
    })
}

#[tauri::command]
pub fn submit_my_assignment(
    state: State<'_, AppState>,
    input: SubmitMyAssignmentInput,
) -> Result<AssignmentSubmission, String> {
    let user = require_student(&state)?;
    if input.body.trim().is_empty() && input.file_name.as_deref().unwrap_or("").trim().is_empty() {
        return Err("Enter text or attach a file".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let course_id: String = conn
        .query_row(
            "SELECT course_id FROM assignments WHERE id = ?1",
            params![input.assignment_id],
            |row| row.get(0),
        )
        .map_err(|_| "Assignment not found".to_string())?;
    let enrolled: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM enrollments WHERE course_id = ?1 AND student_id = ?2 AND status = 'active'",
            params![course_id, user.id],
            |row| row.get(0),
        )
        .map(|c: i64| c > 0)
        .unwrap_or(false);
    if !enrolled {
        return Err("You are not enrolled in this course".into());
    }
    crate::commands::submissions::submit_assignment_work(
        &conn,
        crate::models::SubmitAssignmentInput {
            assignment_id: input.assignment_id,
            student_name: user.name.clone(),
            body: input.body,
            file_name: input.file_name,
            file_data: input.file_data,
        },
        Some(user.id),
    )
}
