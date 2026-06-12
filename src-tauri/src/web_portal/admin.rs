use super::{bad_request, db_err, require_web_user, ErrorResponse, HttpServerState};
use crate::commands::enrollments::{
    enroll_student_work, list_enrollments_for_course, list_students_for_school, unenroll_student_work,
};
use crate::commands::sync::{read_public_base_url, read_public_hub_path, set_setting_work};
use crate::commands::{
    build_dashboard_stats, create_course_work, create_user_work, list_courses_for_school,
    list_users_for_school,
};
use crate::commands::tenancy::{require_org_admin, resolve_active_school};
use crate::models::{
    Course, CreateCourseInput, CreateUserInput, DashboardStats, EnrollStudentInput, Enrollment,
    User,
};
use axum::{
    extract::{Query, State as AxumState},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CourseIdQuery {
    pub course_id: String,
}

#[derive(Deserialize)]
pub struct EnrollmentIdBody {
    pub enrollment_id: String,
}

#[derive(Serialize)]
pub struct AdminSettings {
    pub school_name: String,
    pub public_base_url: Option<String>,
    pub public_hub_path: String,
}

#[derive(Deserialize)]
pub struct SaveAdminSettingsInput {
    pub public_base_url: Option<String>,
    pub public_hub_path: Option<String>,
}

pub fn routes() -> Router<HttpServerState> {
    Router::new()
        .route("/api/web/admin/dashboard", get(admin_dashboard))
        .route("/api/web/admin/users", get(admin_list_users).post(admin_create_user))
        .route("/api/web/admin/courses", get(admin_list_courses).post(admin_create_course))
        .route("/api/web/admin/students", get(admin_list_students))
        .route(
            "/api/web/admin/enrollments",
            get(admin_list_enrollments).post(admin_enroll_student),
        )
        .route("/api/web/admin/enrollments/unenroll", post(admin_unenroll))
        .route(
            "/api/web/admin/settings",
            get(admin_get_settings).post(admin_save_settings),
        )
}

fn require_admin(user: &User) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    require_org_admin(user).map_err(|e| bad_request(e))
}

fn ensure_admin_course(
    conn: &rusqlite::Connection,
    user: &User,
    course_id: &str,
) -> Result<(), String> {
    let school_id = resolve_active_school(conn, &user.id)?;
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
    Ok(())
}

async fn admin_dashboard(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<DashboardStats>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let school_id = resolve_active_school(&conn, &user.id).map_err(|e| bad_request(e))?;
    let stats = build_dashboard_stats(&conn, &school_id).map_err(|e| bad_request(e))?;
    Ok(Json(stats))
}

async fn admin_list_users(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let school_id = resolve_active_school(&conn, &user.id).map_err(|e| bad_request(e))?;
    Ok(Json(
        list_users_for_school(&conn, &school_id).map_err(|e| bad_request(e))?,
    ))
}

async fn admin_create_user(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let school_id = resolve_active_school(&conn, &user.id).map_err(|e| bad_request(e))?;
    Ok(Json(
        create_user_work(&conn, &school_id, &input).map_err(|e| bad_request(e))?,
    ))
}

async fn admin_list_courses(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Course>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let school_id = resolve_active_school(&conn, &user.id).map_err(|e| bad_request(e))?;
    Ok(Json(
        list_courses_for_school(&conn, &school_id).map_err(|e| bad_request(e))?,
    ))
}

async fn admin_create_course(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(input): Json<CreateCourseInput>,
) -> Result<Json<Course>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let school_id = resolve_active_school(&conn, &user.id).map_err(|e| bad_request(e))?;
    Ok(Json(
        create_course_work(&conn, &school_id, &input).map_err(|e| bad_request(e))?,
    ))
}

async fn admin_list_students(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let school_id = resolve_active_school(&conn, &user.id).map_err(|e| bad_request(e))?;
    Ok(Json(
        list_students_for_school(&conn, &school_id).map_err(|e| bad_request(e))?,
    ))
}

async fn admin_list_enrollments(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Query(query): Query<CourseIdQuery>,
) -> Result<Json<Vec<Enrollment>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    ensure_admin_course(&conn, &user, &query.course_id).map_err(|e| bad_request(e))?;
    Ok(Json(
        list_enrollments_for_course(&conn, &query.course_id).map_err(|e| bad_request(e))?,
    ))
}

async fn admin_enroll_student(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(input): Json<EnrollStudentInput>,
) -> Result<Json<Enrollment>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    ensure_admin_course(&conn, &user, &input.course_id).map_err(|e| bad_request(e))?;
    Ok(Json(
        enroll_student_work(&conn, &input).map_err(|e| bad_request(e))?,
    ))
}

async fn admin_unenroll(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(body): Json<EnrollmentIdBody>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let course_id: String = conn
        .query_row(
            "SELECT course_id FROM enrollments WHERE id = ?1",
            params![body.enrollment_id],
            |row| row.get(0),
        )
        .map_err(|_| bad_request("Enrollment not found".into()))?;
    ensure_admin_course(&conn, &user, &course_id).map_err(|e| bad_request(e))?;
    unenroll_student_work(&conn, &body.enrollment_id).map_err(|e| bad_request(e))?;
    Ok(StatusCode::OK)
}

async fn admin_get_settings(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<AdminSettings>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    let school_name = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'school_name'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "ClassMate".into());
    Ok(Json(AdminSettings {
        school_name,
        public_base_url: read_public_base_url(&conn),
        public_hub_path: read_public_hub_path(&conn),
    }))
}

async fn admin_save_settings(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(input): Json<SaveAdminSettingsInput>,
) -> Result<Json<AdminSettings>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["admin"])?;
    require_admin(&user)?;
    let conn = state.db.lock().map_err(|_| db_err())?;
    if let Some(url) = input.public_base_url {
        let trimmed = url.trim().trim_end_matches('/').to_string();
        if trimmed.is_empty() {
            set_setting_work(&conn, "public_base_url", "").map_err(|e| bad_request(e))?;
        } else {
            set_setting_work(&conn, "public_base_url", &trimmed).map_err(|e| bad_request(e))?;
        }
    }
    if let Some(path) = input.public_hub_path {
        let trimmed = if path.trim().is_empty() {
            "/hub".to_string()
        } else {
            path.trim().to_string()
        };
        set_setting_work(&conn, "public_hub_path", &trimmed).map_err(|e| bad_request(e))?;
    }
    let school_name = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'school_name'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "ClassMate".into());
    Ok(Json(AdminSettings {
        school_name,
        public_base_url: read_public_base_url(&conn),
        public_hub_path: read_public_hub_path(&conn),
    }))
}
