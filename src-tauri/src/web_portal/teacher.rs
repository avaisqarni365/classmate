use super::{bad_request, db_err, ErrorResponse, HttpServerState, require_web_user};
use crate::commands::announcements::{create_announcement_work, list_announcements_for_course};
use crate::commands::gradebook::{build_gradebook, save_grade_work};
use crate::commands::materials::{create_material_work, lectures_for_course};
use crate::commands::tenancy::resolve_active_school;
use crate::models::{
    Announcement, Course, CourseMaterial, CreateAnnouncementInput, CreateMaterialInput,
    Gradebook, MaterialWithAiLab, SaveGradeInput, User,
};
use axum::{
    extract::{Query, State as AxumState},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use rusqlite::params;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CourseIdQuery {
    pub course_id: String,
}

#[derive(Deserialize)]
pub struct AnnouncementIdBody {
    pub id: String,
}

#[derive(Deserialize)]
pub struct MaterialIdBody {
    pub material_id: String,
}

pub fn routes() -> Router<HttpServerState> {
    Router::new()
        .route("/api/web/teacher/courses", get(teacher_courses))
        .route("/api/web/teacher/gradebook", get(teacher_gradebook))
        .route("/api/web/teacher/grade", post(teacher_save_grade))
        .route("/api/web/teacher/announcements", get(teacher_list_announcements).post(teacher_create_announcement))
        .route("/api/web/teacher/announcements/delete", post(teacher_delete_announcement))
        .route("/api/web/teacher/lectures", get(teacher_lectures))
        .route("/api/web/teacher/materials", post(teacher_create_material))
        .route("/api/web/teacher/materials/delete", post(teacher_delete_material))
}

fn ensure_course_access(
    conn: &rusqlite::Connection,
    user: &User,
    course_id: &str,
) -> Result<(), String> {
    let school_id = resolve_active_school(conn, &user.id)?;
    let (teacher_id, course_school): (Option<String>, String) = conn
        .query_row(
            "SELECT teacher_id, school_id FROM courses WHERE id = ?1",
            params![course_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Course not found".to_string())?;
    if course_school != school_id {
        return Err("Course not in your school".into());
    }
    if user.role == "admin" {
        return Ok(());
    }
    if user.role == "teacher" && teacher_id.as_deref() == Some(user.id.as_str()) {
        return Ok(());
    }
    Err("You do not have access to this course".into())
}

fn list_teacher_courses(conn: &rusqlite::Connection, user: &User) -> Result<Vec<Course>, String> {
    let school_id = resolve_active_school(conn, &user.id)?;
    let sql = if user.role == "admin" {
        "SELECT c.id, c.title, c.code, c.description, c.teacher_id, u.name,
                c.term, COUNT(e.id) AS student_count, c.created_at
         FROM courses c
         LEFT JOIN users u ON u.id = c.teacher_id
         LEFT JOIN enrollments e ON e.course_id = c.id AND e.status = 'active'
         WHERE c.school_id = ?1
         GROUP BY c.id
         ORDER BY c.title COLLATE NOCASE"
    } else {
        "SELECT c.id, c.title, c.code, c.description, c.teacher_id, u.name,
                c.term, COUNT(e.id) AS student_count, c.created_at
         FROM courses c
         LEFT JOIN users u ON u.id = c.teacher_id
         LEFT JOIN enrollments e ON e.course_id = c.id AND e.status = 'active'
         WHERE c.school_id = ?1 AND c.teacher_id = ?2
         GROUP BY c.id
         ORDER BY c.title COLLATE NOCASE"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let map_row = |row: &rusqlite::Row<'_>| {
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
    };

    let courses = if user.role == "admin" {
        stmt.query_map(params![school_id], map_row)
    } else {
        stmt.query_map(params![school_id, user.id], map_row)
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(courses)
}

async fn teacher_courses(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Course>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let courses = list_teacher_courses(&conn, &user).map_err(|e| bad_request(e))?;
    Ok(Json(courses))
}

async fn teacher_gradebook(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Query(query): Query<CourseIdQuery>,
) -> Result<Json<Gradebook>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    ensure_course_access(&conn, &user, &query.course_id).map_err(|e| bad_request(e))?;
    let book = build_gradebook(&conn, query.course_id).map_err(|e| bad_request(e))?;
    Ok(Json(book))
}

async fn teacher_save_grade(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(input): Json<SaveGradeInput>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let course_id: String = conn
        .query_row(
            "SELECT course_id FROM assignments WHERE id = ?1",
            params![input.assignment_id],
            |row| row.get(0),
        )
        .map_err(|_| bad_request("Assignment not found".into()))?;
    ensure_course_access(&conn, &user, &course_id).map_err(|e| bad_request(e))?;
    save_grade_work(&conn, &input).map_err(|e| bad_request(e))?;
    Ok(StatusCode::OK)
}

async fn teacher_list_announcements(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Query(query): Query<CourseIdQuery>,
) -> Result<Json<Vec<Announcement>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    ensure_course_access(&conn, &user, &query.course_id).map_err(|e| bad_request(e))?;
    let rows = list_announcements_for_course(&conn, Some(&query.course_id)).map_err(|e| bad_request(e))?;
    Ok(Json(rows))
}

async fn teacher_create_announcement(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(input): Json<CreateAnnouncementInput>,
) -> Result<Json<Announcement>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    if let Some(ref cid) = input.course_id {
        ensure_course_access(&conn, &user, cid).map_err(|e| bad_request(e))?;
    } else if user.role != "admin" {
        return Err(bad_request("Course is required".into()));
    }
    let announcement =
        create_announcement_work(&conn, &input, Some(&user.id)).map_err(|e| bad_request(e))?;
    Ok(Json(announcement))
}

async fn teacher_delete_announcement(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(body): Json<AnnouncementIdBody>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let (course_id, author_id): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT course_id, author_id FROM announcements WHERE id = ?1",
            params![body.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| bad_request("Announcement not found".into()))?;
    if let Some(ref cid) = course_id {
        ensure_course_access(&conn, &user, cid).map_err(|e| bad_request(e))?;
    } else if user.role != "admin" {
        return Err(bad_request("Cannot delete this announcement".into()));
    }
    if user.role == "teacher" && author_id.as_deref() != Some(user.id.as_str()) {
        return Err(bad_request("You can only delete your own announcements".into()));
    }
    conn.execute("DELETE FROM announcements WHERE id = ?1", params![body.id])
        .map_err(|_| db_err())?;
    Ok(StatusCode::OK)
}

async fn teacher_lectures(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Query(query): Query<CourseIdQuery>,
) -> Result<Json<Vec<MaterialWithAiLab>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    ensure_course_access(&conn, &user, &query.course_id).map_err(|e| bad_request(e))?;
    let lectures = lectures_for_course(&conn, &query.course_id).map_err(|e| bad_request(e))?;
    Ok(Json(lectures))
}

async fn teacher_create_material(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(input): Json<CreateMaterialInput>,
) -> Result<Json<CourseMaterial>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    ensure_course_access(&conn, &user, &input.course_id).map_err(|e| bad_request(e))?;
    let material = create_material_work(&conn, &input).map_err(|e| bad_request(e))?;
    Ok(Json(material))
}

async fn teacher_delete_material(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(body): Json<MaterialIdBody>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let course_id: String = conn
        .query_row(
            "SELECT course_id FROM course_materials WHERE id = ?1",
            params![body.material_id],
            |row| row.get(0),
        )
        .map_err(|_| bad_request("Material not found".into()))?;
    ensure_course_access(&conn, &user, &course_id).map_err(|e| bad_request(e))?;
    conn.execute(
        "DELETE FROM course_materials WHERE id = ?1",
        params![body.material_id],
    )
    .map_err(|_| db_err())?;
    Ok(StatusCode::OK)
}
