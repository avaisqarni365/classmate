use super::{bad_request, db_err, require_web_user, ErrorResponse, HttpServerState};
use crate::commands::ai_lab::artizai_config;
use crate::web_portal::notes::student_lectures;
use crate::commands::tenancy::resolve_active_school;
use crate::models::{MaterialWithAiLab, User};
use axum::{
    extract::{Query, State as AxumState},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use rusqlite::params;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CourseIdQuery {
    pub course_id: String,
}

pub fn routes() -> Router<HttpServerState> {
    Router::new()
        .route("/api/web/artizai/config", get(web_artizai_config))
        .route("/api/web/student/lectures", get(web_student_lectures))
}

async fn web_artizai_config(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<crate::commands::ai_lab::ArtizAiConfig>, (StatusCode, Json<ErrorResponse>)> {
    let _user = require_web_user(&state, &headers, &["student", "parent", "teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    Ok(Json(artizai_config(&conn)))
}

async fn web_student_lectures(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Query(query): Query<CourseIdQuery>,
) -> Result<Json<Vec<MaterialWithAiLab>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["student"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let school_id = resolve_active_school(&conn, &user.id).map_err(|e| bad_request(e))?;
    ensure_student_course(&conn, &user, &school_id, &query.course_id).map_err(|e| bad_request(e))?;
    let lectures = student_lectures(&conn, &user.id, &school_id, &query.course_id)
        .map_err(|e| bad_request(e))?;
    Ok(Json(lectures))
}

fn ensure_student_course(
    conn: &rusqlite::Connection,
    user: &User,
    school_id: &str,
    course_id: &str,
) -> Result<(), String> {
    let course_school: String = conn
        .query_row(
            "SELECT school_id FROM courses WHERE id = ?1",
            params![course_id],
            |row| row.get(0),
        )
        .map_err(|_| "Course not found".to_string())?;
    if course_school != school_id {
        return Err("Course not in your school".into());
    }
    let enrolled: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM enrollments
             WHERE course_id = ?1 AND student_id = ?2 AND status = 'active'",
            params![course_id, user.id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if enrolled == 0 {
        return Err("You are not enrolled in this course".into());
    }
    Ok(())
}
