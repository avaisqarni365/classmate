mod admin;
mod lectures;
mod notes;
mod teacher;

use crate::commands::parent::build_parent_dashboard;
use crate::commands::student::build_student_dashboard;
use crate::commands::tenancy::resolve_active_school;
use crate::models::{LoginInput, ParentGradeEntry, ParentStudentSummary, StudentDashboard, User};
use axum::{
    extract::State as AxumState,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use bcrypt::verify;
use chrono::{Duration, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

const SESSION_COOKIE: &str = "classmate_web_session";
const SESSION_HOURS: i64 = 24;

#[derive(Clone)]
pub struct HttpServerState {
    pub db: Arc<Mutex<Connection>>,
    pub sync_token: String,
    pub web_sessions: Arc<Mutex<HashMap<String, WebSession>>>,
}

impl HttpServerState {
    pub fn new(db: Arc<Mutex<Connection>>, sync_token: String) -> Self {
        Self {
            db,
            sync_token,
            web_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Clone)]
struct WebSession {
    user: User,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
struct LoginBody {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    user: User,
    role: String,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    error: String,
}

pub fn site_routes() -> Router<HttpServerState> {
    Router::new()
        .merge(notes::routes())
        .merge(lectures::routes())
        .merge(admin::routes())
        .merge(teacher::routes())
        .route("/", get(landing_page))
        .route("/portal", get(portal_page))
        .route("/notes/pad", get(notes_pad_page))
        .route("/api/web/login", post(web_login))
        .route("/api/web/logout", post(web_logout))
        .route("/api/web/me", get(web_me))
        .route("/api/web/student/dashboard", get(student_dashboard))
        .route("/api/web/parent/dashboard", get(parent_dashboard))
        .route("/api/web/parent/grades", get(parent_grades))
}

async fn landing_page() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../../public_pages/index.html"))
}

async fn portal_page() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../../public_pages/portal.html"))
}

async fn notes_pad_page() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../../public_pages/notes-pad.html"))
}

fn authenticate_user(conn: &Connection, input: &LoginInput) -> Result<User, String> {
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
    let valid = verify(&input.password, &hash).map_err(|e| e.to_string())?;
    if !valid {
        return Err("Invalid email or password".into());
    }
    Ok(user)
}

fn session_from_cookie(headers: &HeaderMap, sessions: &HashMap<String, WebSession>) -> Option<User> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    let token = cookie
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix(&format!("{SESSION_COOKIE}=")))?;
    let session = sessions.get(token)?;
    if session.expires_at < Utc::now() {
        return None;
    }
    Some(session.user.clone())
}

fn set_session_cookie(token: &str) -> String {
    format!(
        "{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}; Secure",
        SESSION_HOURS * 3600
    )
}

fn clear_session_cookie() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0; Secure")
}

async fn web_login(
    AxumState(state): AxumState<HttpServerState>,
    Json(body): Json<LoginBody>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let input = LoginInput {
        email: body.email,
        password: body.password,
    };
    let user = {
        let conn = state.db.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".into(),
                }),
            )
        })?;
        authenticate_user(&conn, &input).map_err(|msg| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse { error: msg }),
            )
        })?
    };

    if !matches!(
        user.role.as_str(),
        "student" | "parent" | "teacher" | "admin"
    ) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Unsupported role for web login.".into(),
            }),
        ));
    }

    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(SESSION_HOURS);
    {
        let mut sessions = state.web_sessions.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Session error".into(),
                }),
            )
        })?;
        sessions.insert(
            token.clone(),
            WebSession {
                user: user.clone(),
                expires_at,
            },
        );
    }

    let mut response = Json(LoginResponse {
        role: user.role.clone(),
        user: user.clone(),
    })
    .into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&set_session_cookie(&token)).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Cookie error".into(),
                }),
            )
        })?,
    );
    Ok(response)
}

async fn web_logout(AxumState(state): AxumState<HttpServerState>, headers: HeaderMap) -> Response {
    if let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        if let Some(token) = cookie
            .split(';')
            .map(str::trim)
            .find_map(|part| part.strip_prefix(&format!("{SESSION_COOKIE}=")))
        {
            if let Ok(mut sessions) = state.web_sessions.lock() {
                sessions.remove(token);
            }
        }
    }
    let mut response = StatusCode::OK.into_response();
    if let Ok(value) = axum::http::HeaderValue::from_str(&clear_session_cookie()) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
    response
}

async fn web_me(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let sessions = state.web_sessions.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Session error".into(),
            }),
        )
    })?;
    let user = session_from_cookie(&headers, &sessions).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Not logged in".into(),
        }),
    ))?;
    Ok(Json(user))
}

pub(crate) fn require_web_user(
    state: &HttpServerState,
    headers: &HeaderMap,
    allowed_roles: &[&str],
) -> Result<User, (StatusCode, Json<ErrorResponse>)> {
    let sessions = state.web_sessions.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Session error".into(),
            }),
        )
    })?;
    let user = session_from_cookie(headers, &sessions).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Not logged in".into(),
        }),
    ))?;
    if !allowed_roles.contains(&user.role.as_str()) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".into(),
            }),
        ));
    }
    Ok(user)
}

async fn student_dashboard(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<StudentDashboard>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["student"])?;
    let conn = state.db.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".into(),
            }),
        )
    })?;
    let school_id = resolve_active_school(&conn, &user.id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let dashboard = build_student_dashboard(&conn, &user.id, &school_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(dashboard))
}

async fn parent_dashboard(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ParentStudentSummary>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["parent"])?;
    let conn = state.db.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".into(),
            }),
        )
    })?;
    let dashboard = build_parent_dashboard(&conn, &user.id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(dashboard))
}

async fn parent_grades(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ParentGradeEntry>>, (StatusCode, Json<ErrorResponse>)> {
    let user = require_web_user(&state, &headers, &["parent"])?;
    let conn = state.db.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".into(),
            }),
        )
    })?;
    let grades = parent_grades_query(&conn, &user.id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(grades))
}

fn parent_grades_query(conn: &Connection, parent_id: &str) -> Result<Vec<ParentGradeEntry>, String> {
    use crate::commands::rubrics::rubric_scores_display;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.name, c.title, a.id, a.title, g.points, a.max_points, g.feedback, g.graded_at, g.rubric_scores_json
             FROM parent_links pl
             JOIN users u ON u.id = pl.student_id
             JOIN enrollments e ON e.student_id = u.id AND e.status = 'active'
             JOIN courses c ON c.id = e.course_id
             JOIN assignments a ON a.course_id = c.id
             LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = u.id
             WHERE pl.parent_id = ?1 AND g.points IS NOT NULL
             ORDER BY g.graded_at DESC, a.title ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![parent_id], |row| {
            let assignment_id: String = row.get(3)?;
            let rubric_scores_json: Option<String> = row.get(9)?;
            let rubric_scores = rubric_scores_display(
                conn,
                &assignment_id,
                rubric_scores_json.as_deref(),
            )
            .ok()
            .flatten();
            Ok(ParentGradeEntry {
                student_id: row.get(0)?,
                student_name: row.get(1)?,
                course_title: row.get(2)?,
                assignment_id,
                assignment_title: row.get(4)?,
                points: row.get(5)?,
                max_points: row.get(6)?,
                feedback: row.get(7)?,
                graded_at: row.get(8)?,
                rubric_scores,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub(crate) fn db_err() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Database error".into(),
        }),
    )
}

pub(crate) fn bad_request(error: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse { error }),
    )
}
