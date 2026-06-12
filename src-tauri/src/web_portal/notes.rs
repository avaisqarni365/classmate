use super::{bad_request, db_err, require_web_user, ErrorResponse, HttpServerState};
use crate::commands::materials::lectures_for_course_student;
use crate::commands::notes::{
    attach_capture_session_work, create_capture_session_work, load_capture_session,
    update_capture_ink_work,
};
use crate::models::{
    CaptureSession, CourseMaterial, CreateCaptureSessionInput, MaterialLabCompletion,
    UpdateCaptureInkInput,
};
use axum::{
    extract::{Path, State as AxumState},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use rusqlite::params;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct OptionalTitleBody {
    pub title: Option<String>,
}

pub fn routes() -> Router<HttpServerState> {
    Router::new()
        .route("/api/web/teacher/capture-session", post(web_create_capture))
        .route("/api/web/capture/{session_id}", get(web_get_capture).post(web_update_capture))
        .route("/api/web/teacher/capture/{session_id}/attach", post(web_attach_capture))
        .route("/api/web/student/lab-complete", post(web_mark_lab_complete))
}

async fn web_create_capture(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(input): Json<CreateCaptureSessionInput>,
) -> Result<Json<CaptureSession>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    Ok(Json(
        create_capture_session_work(&conn, &input.course_id, &user.id, &input.title)
            .map_err(|e| bad_request(e))?,
    ))
}

async fn web_get_capture(
    AxumState(state): AxumState<HttpServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<CaptureSession>, (StatusCode, Json<ErrorResponse>)> {
    let conn = state.db.lock().map_err(|_| db_err())?;
    Ok(Json(
        load_capture_session(&conn, &session_id).map_err(|e| bad_request(e))?,
    ))
}

async fn web_update_capture(
    AxumState(state): AxumState<HttpServerState>,
    Path(session_id): Path<String>,
    Json(body): Json<UpdateCaptureInkInput>,
) -> Result<Json<CaptureSession>, (StatusCode, Json<ErrorResponse>)> {
    let conn = state.db.lock().map_err(|_| db_err())?;
    let input = UpdateCaptureInkInput {
        session_id,
        ink_json: body.ink_json,
        preview_data_url: body.preview_data_url,
    };
    Ok(Json(
        update_capture_ink_work(&conn, &input).map_err(|e| bad_request(e))?,
    ))
}

async fn web_attach_capture(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(body): Json<OptionalTitleBody>,
) -> Result<Json<CourseMaterial>, (StatusCode, Json<ErrorResponse>)> {
    let _user = require_web_user(&state, &headers, &["teacher", "admin"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    Ok(Json(
        attach_capture_session_work(&conn, &session_id, body.title).map_err(|e| bad_request(e))?,
    ))
}

#[derive(Deserialize)]
struct LabCompleteBody {
    material_id: String,
}

async fn web_mark_lab_complete(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(body): Json<LabCompleteBody>,
) -> Result<Json<MaterialLabCompletion>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["student"])?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO material_lab_completions (id, material_id, student_id, completed_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(material_id, student_id) DO UPDATE SET completed_at = excluded.completed_at",
        params![id, body.material_id, user.id, now],
    )
    .map_err(|_| db_err())?;
    Ok(Json(MaterialLabCompletion {
        material_id: body.material_id,
        student_id: user.id,
        completed_at: now,
    }))
}

pub fn student_lectures(
    conn: &rusqlite::Connection,
    user_id: &str,
    school_id: &str,
    course_id: &str,
) -> Result<Vec<crate::models::MaterialWithAiLab>, String> {
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
    let enrolled: i64 = conn.query_row(
        "SELECT COUNT(*) FROM enrollments
         WHERE course_id = ?1 AND student_id = ?2 AND status = 'active'",
        params![course_id, user_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if enrolled == 0 {
        return Err("You are not enrolled in this course".into());
    }
    lectures_for_course_student(conn, course_id, Some(user_id))
}
