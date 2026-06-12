use crate::models::{
    HubStatus, QuizAnswerInput, StudentAssignmentView, StudentJoinInfo, StudentMaterialView,
    StudentQuizQuestion, SubmitAssignmentInput, SubmitQuizInput, SubmitQuizResult, VotePollInput,
};
use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

const HUB_PORT: u16 = 8765;

pub struct HubRuntime {
    running: bool,
    port: u16,
    session_id: Option<String>,
    course_id: Option<String>,
    course_title: Option<String>,
    pin: Option<String>,
    video_url: Option<String>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl HubRuntime {
    pub fn new() -> Self {
        Self {
            running: false,
            port: HUB_PORT,
            session_id: None,
            course_id: None,
            course_title: None,
            pin: None,
            video_url: None,
            shutdown: None,
        }
    }

    pub fn status(&self) -> HubStatus {
        let local_ip = local_ip_address::local_ip().ok().map(|ip| ip.to_string());
        let join_url = local_ip.as_ref().map(|ip| format!("http://{}:{}/student", ip, self.port));

        HubStatus {
            running: self.running,
            port: self.port,
            local_ip,
            join_url,
            session_id: self.session_id.clone(),
            course_title: self.course_title.clone(),
            pin: self.pin.clone(),
            video_url: self.video_url.clone(),
            video_running: self.video_url.is_some(),
        }
    }

    pub fn start(
        &mut self,
        db: Arc<Mutex<Connection>>,
        session_id: String,
        course_id: String,
        course_title: String,
        pin: String,
        video_url: Option<String>,
    ) -> Result<(), String> {
        if self.running {
            self.stop();
        }

        let (tx, rx) = oneshot::channel();
        let db_clone = db.clone();
        let session_for_routes = session_id.clone();
        let course_id_for_routes = course_id.clone();
        let pin_for_routes = pin.clone();
        let course_for_routes = course_title.clone();

        let video_for_routes = video_url.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                let app_state = HubServerState {
                    db: db_clone,
                    session_id: session_for_routes,
                    course_id: course_id_for_routes,
                    pin: pin_for_routes,
                    course_title: course_for_routes,
                    video_url: video_for_routes,
                };

                let app = Router::new()
                    .route("/", get(student_portal))
                    .route("/student", get(student_portal))
                    .route("/sw.js", get(service_worker))
                    .route("/api/hub/info", get(hub_info))
                    .route("/api/hub/join", post(hub_join))
                    .route("/api/student/materials", get(student_materials))
                    .route("/api/student/assignments", get(student_assignments))
                    .route("/api/student/announcements", get(student_announcements))
                    .route("/api/student/forum", get(student_forum))
                    .route("/api/student/forum/post", post(student_forum_post))
                    .route("/api/student/quizzes", get(student_quizzes))
                    .route("/api/student/quiz/{quiz_id}", get(student_quiz_detail))
                    .route("/api/student/quiz/submit", post(student_quiz_submit))
                    .route("/api/student/poll", get(student_poll))
                    .route("/api/student/poll/vote", post(student_poll_vote))
                    .route("/api/student/submit", post(student_submit_assignment))
                    .route("/api/student/push/register", post(student_push_register))
                    .with_state(app_state)
                    .layer(CorsLayer::permissive());

                let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", HUB_PORT))
                    .await
                    .expect("Failed to bind hub port");

                axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = rx.await;
                    })
                    .await
                    .ok();
            });
        });

        self.running = true;
        self.session_id = Some(session_id);
        self.course_id = Some(course_id);
        self.course_title = Some(course_title);
        self.pin = Some(pin);
        self.video_url = video_url;
        self.shutdown = Some(tx);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        self.running = false;
        self.session_id = None;
        self.course_id = None;
        self.course_title = None;
        self.pin = None;
        self.video_url = None;
    }
}

#[derive(Clone)]
struct HubServerState {
    db: Arc<Mutex<Connection>>,
    session_id: String,
    course_id: String,
    pin: String,
    course_title: String,
    video_url: Option<String>,
}

#[derive(Serialize)]
struct HubInfoResponse {
    course_title: String,
    session_id: String,
    requires_pin: bool,
    video_url: Option<String>,
}

#[derive(Deserialize)]
struct JoinRequest {
    pin: String,
    name: String,
}

async fn hub_info(AxumState(state): AxumState<HubServerState>) -> Json<HubInfoResponse> {
    Json(HubInfoResponse {
        course_title: state.course_title.clone(),
        session_id: state.session_id.clone(),
        requires_pin: true,
        video_url: state.video_url.clone(),
    })
}

async fn hub_join(
    AxumState(state): AxumState<HubServerState>,
    Json(body): Json<JoinRequest>,
) -> Result<Json<StudentJoinInfo>, StatusCode> {
    if body.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if body.pin != state.pin {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let now = Utc::now().to_rfc3339();
    let attendance_id = Uuid::new_v4().to_string();

    {
        let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        conn.execute(
            "INSERT INTO attendance (id, session_id, student_name, joined_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![attendance_id, state.session_id, body.name.trim(), now],
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(StudentJoinInfo {
        session_id: state.session_id.clone(),
        course_title: state.course_title.clone(),
        message: format!("Welcome, {}! You are checked in.", body.name.trim()),
    }))
}

async fn student_materials(
    AxumState(state): AxumState<HubServerState>,
) -> Result<Json<Vec<StudentMaterialView>>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let lectures = crate::commands::materials::lectures_for_course(&conn, &state.course_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        lectures
            .into_iter()
            .map(|entry| StudentMaterialView {
                id: entry.material.id,
                title: entry.material.title,
                kind: entry.material.kind,
                content: entry.material.content,
                ai_lab: entry.ai_lab,
            })
            .collect(),
    ))
}

async fn student_assignments(
    AxumState(state): AxumState<HubServerState>,
) -> Result<Json<Vec<StudentAssignmentView>>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, due_at, max_points FROM assignments
             WHERE course_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = stmt
        .query_map(params![state.course_id], |row| {
            Ok(StudentAssignmentView {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                due_at: row.get(3)?,
                max_points: row.get(4)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

#[derive(Serialize)]
struct AnnouncementView {
    title: String,
    body: String,
    created_at: String,
}

async fn student_announcements(
    AxumState(state): AxumState<HubServerState>,
) -> Result<Json<Vec<AnnouncementView>>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn
        .prepare(
            "SELECT title, body, created_at FROM announcements
             WHERE course_id = ?1 OR course_id IS NULL
             ORDER BY created_at DESC LIMIT 20",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map(params![state.course_id], |row| {
            Ok(AnnouncementView {
                title: row.get(0)?,
                body: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows))
}

#[derive(Serialize)]
struct ForumView {
    id: String,
    title: String,
    author_name: String,
    created_at: String,
}

#[derive(Deserialize)]
struct ForumPostBody {
    topic_id: String,
    author_name: String,
    body: String,
}

async fn student_forum(
    AxumState(state): AxumState<HubServerState>,
) -> Result<Json<Vec<ForumView>>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, author_name, created_at FROM forum_topics
             WHERE course_id = ?1 ORDER BY created_at DESC LIMIT 30",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map(params![state.course_id], |row| {
            Ok(ForumView {
                id: row.get(0)?,
                title: row.get(1)?,
                author_name: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows))
}

async fn student_forum_post(
    AxumState(state): AxumState<HubServerState>,
    Json(body): Json<ForumPostBody>,
) -> Result<StatusCode, StatusCode> {
    if body.author_name.trim().is_empty() || body.body.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let now = Utc::now().to_rfc3339();
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    conn.execute(
        "INSERT INTO forum_posts (id, topic_id, author_name, body, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            Uuid::new_v4().to_string(),
            body.topic_id,
            body.author_name.trim(),
            body.body.trim(),
            now
        ],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::CREATED)
}

#[derive(Serialize)]
struct StudentQuizListItem {
    id: String,
    title: String,
    description: Option<String>,
    question_count: i64,
    max_points: f64,
    time_limit_minutes: Option<i64>,
}

async fn student_quizzes(
    AxumState(state): AxumState<HubServerState>,
) -> Result<Json<Vec<StudentQuizListItem>>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.title, q.description, COUNT(qq.id), q.max_points, q.time_limit_minutes
             FROM quizzes q
             LEFT JOIN quiz_questions qq ON qq.quiz_id = q.id
             WHERE q.course_id = ?1 AND q.status = 'published'
             GROUP BY q.id
             ORDER BY q.created_at DESC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map(params![state.course_id], |row| {
            Ok(StudentQuizListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                question_count: row.get(3)?,
                max_points: row.get(4)?,
                time_limit_minutes: row.get(5)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows))
}

#[derive(Serialize)]
struct StudentQuizDetailResponse {
    id: String,
    title: String,
    description: Option<String>,
    time_limit_minutes: Option<i64>,
    questions: Vec<StudentQuizQuestion>,
}

async fn student_quiz_detail(
    AxumState(state): AxumState<HubServerState>,
    Path(quiz_id): Path<String>,
) -> Result<Json<StudentQuizDetailResponse>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let (id, title, description, time_limit): (String, String, Option<String>, Option<i64>) = conn
        .query_row(
            "SELECT id, title, description, time_limit_minutes FROM quizzes
             WHERE id = ?1 AND course_id = ?2 AND status = 'published'",
            params![quiz_id, state.course_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, prompt, kind, options_json, points FROM quiz_questions
             WHERE quiz_id = ?1 ORDER BY sort_order ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let questions = stmt
        .query_map(params![quiz_id], |row| {
            let options_json: String = row.get(3)?;
            let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
            Ok(StudentQuizQuestion {
                id: row.get(0)?,
                prompt: row.get(1)?,
                kind: row.get(2)?,
                options,
                points: row.get(4)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(StudentQuizDetailResponse {
        id,
        title,
        description,
        time_limit_minutes: time_limit,
        questions,
    }))
}

#[derive(Deserialize)]
struct HubQuizSubmitBody {
    quiz_id: String,
    student_name: String,
    answers: Vec<QuizAnswerInput>,
}

async fn student_quiz_submit(
    AxumState(state): AxumState<HubServerState>,
    Json(body): Json<HubQuizSubmitBody>,
) -> Result<Json<SubmitQuizResult>, StatusCode> {
    if body.student_name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let belongs: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM quizzes WHERE id = ?1 AND course_id = ?2 AND status = 'published'",
            params![body.quiz_id, state.course_id],
            |row| row.get(0),
        )
        .map(|c: i64| c > 0)
        .unwrap_or(false);
    if !belongs {
        return Err(StatusCode::NOT_FOUND);
    }

    crate::commands::quizzes::grade_quiz_submission(
        &conn,
        SubmitQuizInput {
            quiz_id: body.quiz_id,
            student_name: body.student_name,
            answers: body.answers,
        },
    )
    .map(Json)
    .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn student_poll(
    AxumState(state): AxumState<HubServerState>,
) -> Result<Json<crate::models::PollResults>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let poll = conn
        .query_row(
            "SELECT id, session_id, question, options_json, status, created_at
             FROM session_polls WHERE session_id = ?1 AND status = 'open'
             ORDER BY created_at DESC LIMIT 1",
            params![state.session_id],
            |row| {
                let options_json: String = row.get(3)?;
                let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
                Ok(crate::models::SessionPoll {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    question: row.get(2)?,
                    options,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match poll {
        Some(p) => crate::commands::polls::poll_results(&conn, p)
            .map(Json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Deserialize)]
struct HubPollVoteBody {
    poll_id: String,
    student_name: String,
    option_index: i64,
}

async fn student_poll_vote(
    AxumState(state): AxumState<HubServerState>,
    Json(body): Json<HubPollVoteBody>,
) -> Result<Json<crate::models::PollResults>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let belongs: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM session_polls WHERE id = ?1 AND session_id = ?2",
            params![body.poll_id, state.session_id],
            |row| row.get(0),
        )
        .map(|c: i64| c > 0)
        .unwrap_or(false);
    if !belongs {
        return Err(StatusCode::NOT_FOUND);
    }

    crate::commands::polls::cast_poll_vote(
        &conn,
        VotePollInput {
            poll_id: body.poll_id,
            student_name: body.student_name,
            option_index: body.option_index,
        },
    )
    .map(Json)
    .map_err(|_| StatusCode::BAD_REQUEST)
}

#[derive(Deserialize)]
struct HubPushRegisterBody {
    student_name: String,
    platform: String,
    token: String,
    device_name: Option<String>,
}

async fn student_push_register(
    AxumState(state): AxumState<HubServerState>,
    Json(body): Json<HubPushRegisterBody>,
) -> Result<StatusCode, StatusCode> {
    if body.student_name.trim().is_empty() || body.token.trim().len() < 8 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    crate::commands::push::register_push_device_for_student(
        &conn,
        &state.course_id,
        &body.student_name,
        &body.platform,
        &body.token,
        body.device_name.as_deref(),
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct HubSubmitBody {
    assignment_id: String,
    student_name: String,
    body: String,
    file_name: Option<String>,
    file_data: Option<String>,
}

async fn student_submit_assignment(
    AxumState(state): AxumState<HubServerState>,
    Json(body): Json<HubSubmitBody>,
) -> Result<Json<crate::models::AssignmentSubmission>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let belongs: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM assignments WHERE id = ?1 AND course_id = ?2",
            params![body.assignment_id, state.course_id],
            |row| row.get(0),
        )
        .map(|c: i64| c > 0)
        .unwrap_or(false);
    if !belongs {
        return Err(StatusCode::NOT_FOUND);
    }

    crate::commands::submissions::submit_assignment_work(
        &conn,
        SubmitAssignmentInput {
            assignment_id: body.assignment_id,
            student_name: body.student_name,
            body: body.body,
            file_name: body.file_name,
            file_data: body.file_data,
        },
        None,
    )
    .map(Json)
    .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn service_worker() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        SERVICE_WORKER_JS,
    )
}

const SERVICE_WORKER_JS: &str = r"
self.addEventListener('install', e => { e.waitUntil(caches.open('classmate-v1')); self.skipWaiting(); });
self.addEventListener('fetch', e => {
  if (e.request.method !== 'GET') return;
  e.respondWith(fetch(e.request).catch(() => caches.match(e.request)));
});
";

async fn student_portal() -> impl IntoResponse {
    Html(STUDENT_PORTAL_HTML)
}

const STUDENT_PORTAL_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta name="theme-color" content="#2563eb" />
  <link rel="manifest" href="data:application/json,{&quot;name&quot;:&quot;ClassMate&quot;,&quot;short_name&quot;:&quot;ClassMate&quot;,&quot;display&quot;:&quot;standalone&quot;,&quot;start_url&quot;:&quot;/student&quot;}" />
  <script>if('serviceWorker' in navigator) navigator.serviceWorker.register('/sw.js');</script>
  <title>ClassMate Student</title>
  <style>
    :root { font-family: system-ui, sans-serif; color: #0f172a; background: #f8fafc; }
    body { margin: 0; min-height: 100vh; padding: 1rem; }
    .wrap { max-width: 480px; margin: 0 auto; }
    .card { background: #fff; border: 1px solid #e2e8f0; border-radius: 16px; padding: 1.25rem; margin-bottom: 1rem; box-shadow: 0 8px 24px rgba(15,23,42,.06); }
    h1 { margin: 0 0 .25rem; font-size: 1.35rem; }
    h2 { margin: 0 0 .75rem; font-size: 1rem; }
    p, li { color: #64748b; }
    label { display: block; font-size: .85rem; font-weight: 600; margin: .85rem 0 .35rem; }
    input { width: 100%; box-sizing: border-box; padding: .7rem .85rem; border: 1px solid #cbd5e1; border-radius: 10px; font-size: 1rem; }
    textarea { width: 100%; box-sizing: border-box; padding: .7rem .85rem; border: 1px solid #cbd5e1; border-radius: 10px; font-size: 1rem; margin-top: .35rem; }
    button { margin-top: 1rem; width: 100%; border: 0; border-radius: 10px; padding: .8rem; font-weight: 700; background: #2563eb; color: #fff; cursor: pointer; }
    .ok { margin-top: 1rem; padding: .75rem; border-radius: 10px; background: #ecfdf5; color: #065f46; }
    .err { margin-top: 1rem; padding: .75rem; border-radius: 10px; background: #fef2f2; color: #991b1b; }
    .hidden { display: none; }
    .item { border-top: 1px solid #e2e8f0; padding: .75rem 0; }
    .item:first-child { border-top: 0; padding-top: 0; }
    .tag { display: inline-block; font-size: .7rem; font-weight: 700; padding: .15rem .45rem; border-radius: 999px; background: #eff6ff; color: #1d4ed8; }
    .btn-video { display: inline-block; margin-top: .5rem; padding: .65rem 1rem; background: #059669; color: #fff; border-radius: 10px; text-decoration: none; font-weight: 700; }
    .btn-quiz { margin-top: .5rem; width: auto; padding: .55rem .85rem; font-size: .9rem; }
    .quiz-q { margin-bottom: 1rem; }
    .quiz-q label { font-weight: 500; }
    .quiz-opt { display: flex; align-items: center; gap: .5rem; margin: .35rem 0; font-weight: 400; }
  </style>
</head>
<body>
  <div class="wrap">
    <div id="join-panel" class="card">
      <h1 id="course">Join class</h1>
      <p>Enter the PIN from your teacher and your name.</p>
      <label for="pin">Class PIN</label>
      <input id="pin" inputmode="numeric" maxlength="6" placeholder="000000" />
      <label for="name">Your name</label>
      <input id="name" placeholder="Jane Doe" />
      <button id="join">Join class</button>
      <div class="err hidden" id="err"></div>
    </div>

    <div id="class-panel" class="hidden">
      <div class="card">
        <h1 id="welcome">ClassMate</h1>
        <p id="welcome-msg"></p>
      </div>
      <div class="card" id="video-card" style="display:none">
        <h2>Live video</h2>
        <p>Join the video classroom on this device.</p>
        <a id="video-link" class="btn-video" href="#" target="_blank">Join video class</a>
      </div>
      <div class="card">
        <h2>Announcements</h2>
        <div id="announcements"><p>Loading...</p></div>
      </div>
      <div class="card">
        <h2>Materials</h2>
        <div id="materials"><p>Loading...</p></div>
      </div>
      <div class="card">
        <h2>Live poll</h2>
        <div id="poll"><p>No active poll.</p></div>
      </div>
      <div class="card">
        <h2>Assignments</h2>
        <div id="assignments"><p>Loading...</p></div>
      </div>
      <div class="card">
        <h2>Quizzes</h2>
        <div id="quizzes"><p>Loading...</p></div>
        <div id="quiz-take" class="hidden"></div>
      </div>
      <div class="card">
        <h2>Discussion</h2>
        <div id="forum"><p>Loading...</p></div>
      </div>
    </div>
  </div>
  <script>
    let hubInfo = {};
    let studentName = localStorage.getItem('classmate_student_name') || '';
    fetch('/api/hub/info').then(r => r.json()).then(d => {
      hubInfo = d;
      document.getElementById('course').textContent = d.course_title || 'Join class';
    }).catch(() => {});

    async function loadQuizzes() {
      const res = await fetch('/api/student/quizzes');
      const quizzes = await res.json();
      const el = document.getElementById('quizzes');
      if (!quizzes.length) { el.innerHTML = '<p>No quizzes available.</p>'; return; }
      el.innerHTML = quizzes.map(q => `
        <div class="item">
          <strong>${q.title}</strong>
          <p>${q.description || ''}</p>
          <p>${q.question_count} questions · ${q.max_points} pts</p>
          <button class="btn-quiz" data-quiz="${q.id}">Take quiz</button>
        </div>`).join('');
      el.querySelectorAll('[data-quiz]').forEach(btn => {
        btn.onclick = () => startQuiz(btn.getAttribute('data-quiz'));
      });
    }

    async function startQuiz(quizId) {
      const res = await fetch('/api/student/quiz/' + quizId);
      if (!res.ok) { alert('Quiz not found'); return; }
      const quiz = await res.json();
      const take = document.getElementById('quiz-take');
      take.classList.remove('hidden');
      take.innerHTML = `
        <h2>${quiz.title}</h2>
        <form id="quiz-form">
          ${quiz.questions.map((q, i) => `
            <div class="quiz-q">
              <label>${i + 1}. ${q.prompt} (${q.points} pts)</label>
              ${q.kind === 'short_answer'
                ? `<textarea id="q_${q.id}" rows="2" required placeholder="Your answer"></textarea>`
                : q.options.map((opt, j) => `
                <label class="quiz-opt">
                  <input type="radio" name="q_${q.id}" value="${j}" required /> ${opt}
                </label>`).join('')}
            </div>`).join('')}
          <button type="submit">Submit quiz</button>
        </form>
        <div class="err hidden" id="quiz-err"></div>`;
      document.getElementById('quiz-form').onsubmit = async (e) => {
        e.preventDefault();
        const answers = quiz.questions.map(q => {
          if (q.kind === 'short_answer') {
            return {
              question_id: q.id,
              text_answer: document.getElementById('q_' + q.id).value.trim()
            };
          }
          return {
            question_id: q.id,
            selected_index: Number(document.querySelector(`input[name="q_${q.id}"]:checked`).value)
          };
        });
        const submitRes = await fetch('/api/student/quiz/submit', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ quiz_id: quizId, student_name: studentName, answers })
        });
        const result = await submitRes.json();
        if (!submitRes.ok) {
          document.getElementById('quiz-err').textContent = result.message || 'Submit failed';
          document.getElementById('quiz-err').classList.remove('hidden');
          return;
        }
        take.innerHTML = `<div class="ok">${result.message}</div>`;
        await loadQuizzes();
      };
    }

    async function loadPoll() {
      const pollEl = document.getElementById('poll');
      try {
        const res = await fetch('/api/student/poll');
        if (!res.ok) { pollEl.innerHTML = '<p>No active poll.</p>'; return; }
        const data = await res.json();
        pollEl.innerHTML = `
          <p><strong>${data.poll.question}</strong></p>
          ${data.poll.options.map((opt, i) => `
            <button class="btn-quiz" data-poll="${data.poll.id}" data-opt="${i}">${opt} (${data.vote_counts[i] || 0})</button>
          `).join('')}`;
        pollEl.querySelectorAll('[data-poll]').forEach(btn => {
          btn.onclick = async () => {
            await fetch('/api/student/poll/vote', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                poll_id: btn.getAttribute('data-poll'),
                student_name: studentName,
                option_index: Number(btn.getAttribute('data-opt'))
              })
            });
            await loadPoll();
          };
        });
      } catch { pollEl.innerHTML = '<p>No active poll.</p>'; }
    }

    async function loadClassContent() {
      const [materialsRes, assignmentsRes, annRes, forumRes] = await Promise.all([
        fetch('/api/student/materials'),
        fetch('/api/student/assignments'),
        fetch('/api/student/announcements'),
        fetch('/api/student/forum')
      ]);
      const materials = await materialsRes.json();
      const assignments = await assignmentsRes.json();
      const announcements = await annRes.json();
      const forum = await forumRes.json();
      await loadQuizzes();
      await loadPoll();
      setInterval(loadPoll, 4000);
      if (hubInfo.video_url) {
        document.getElementById('video-card').style.display = 'block';
        const link = document.getElementById('video-link');
        link.href = hubInfo.video_url;
      }
      const annEl = document.getElementById('announcements');
      if (!announcements.length) annEl.innerHTML = '<p>No announcements.</p>';
      else annEl.innerHTML = announcements.map(a => `<div class="item"><strong>${a.title}</strong><p>${a.body}</p></div>`).join('');
      const matEl = document.getElementById('materials');
      const asgEl = document.getElementById('assignments');
      const forumEl = document.getElementById('forum');
      if (!materials.length) matEl.innerHTML = '<p>No materials yet.</p>';
      else matEl.innerHTML = materials.map(m => {
        let body = m.content;
        if (m.kind === 'link') body = `<a class="link" href="${m.content}" target="_blank">${m.content}</a>`;
        else if (m.kind === 'textbook') {
          try {
            const tb = JSON.parse(m.content);
            const links = [];
            if (tb.read_url) links.push(`<a class="link" href="${tb.read_url}" target="_blank">Read online</a>`);
            if (tb.pdf_url) links.push(`<a class="link" href="${tb.pdf_url}" target="_blank">PDF</a>`);
            body = (tb.notes ? `<p>${tb.notes}</p>` : '') + (links.length ? `<p>${links.join(' · ')}</p>` : '');
          } catch (_) { body = m.content; }
        }
        const tag = m.kind === 'textbook' ? 'OpenStax' : m.kind;
        const lab = m.ai_lab ? `<p style="margin-top:.5rem"><a class="link" href="${m.ai_lab.url}" target="_blank" style="font-weight:600">ARTIZAI ${m.ai_lab.lab.name}</a></p>` : '';
        return `<div class="item"><div><span class="tag">${tag}</span> <strong>${m.title}</strong></div><p>${body}</p>${lab}</div>`;
      }).join('');
      if (!assignments.length) asgEl.innerHTML = '<p>No assignments yet.</p>';
      else asgEl.innerHTML = assignments.map(a => `
        <div class="item">
          <strong>${a.title}</strong>
          <p>${a.description || ''}</p>
          <p>Max points: ${a.max_points}</p>
          <label>Your submission</label>
          <textarea id="sub-${a.id}" rows="3" placeholder="Type your answer here..."></textarea>
          <label style="margin-top:0.5rem;display:block">Attach file (max 2 MB)</label>
          <input type="file" id="sub-file-${a.id}" />
          <button class="btn-quiz" data-asg="${a.id}">Submit work</button>
          <div class="ok hidden" id="sub-ok-${a.id}"></div>
        </div>`).join('');
      asgEl.querySelectorAll('[data-asg]').forEach(btn => {
        btn.onclick = async () => {
          const asgId = btn.getAttribute('data-asg');
          const text = document.getElementById('sub-' + asgId).value.trim();
          const fileInput = document.getElementById('sub-file-' + asgId);
          let file_name = null;
          let file_data = null;
          if (fileInput && fileInput.files && fileInput.files[0]) {
            const file = fileInput.files[0];
            if (file.size > 2 * 1024 * 1024) {
              const okEl = document.getElementById('sub-ok-' + asgId);
              okEl.textContent = 'File exceeds 2 MB limit.';
              okEl.classList.remove('hidden', 'ok');
              okEl.classList.add('err');
              return;
            }
            file_name = file.name;
            file_data = await new Promise((resolve, reject) => {
              const reader = new FileReader();
              reader.onload = () => resolve(String(reader.result).split(',')[1] || '');
              reader.onerror = () => reject(new Error('read failed'));
              reader.readAsDataURL(file);
            });
          }
          if (!text && !file_name) return;
          const res = await fetch('/api/student/submit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ assignment_id: asgId, student_name: studentName, body: text, file_name, file_data })
          });
          const okEl = document.getElementById('sub-ok-' + asgId);
          if (res.ok) {
            okEl.textContent = 'Submitted!';
            okEl.classList.remove('hidden');
          } else {
            okEl.textContent = 'Could not submit.';
            okEl.classList.remove('hidden');
            okEl.classList.remove('ok');
            okEl.classList.add('err');
          }
        };
      });
      if (!forum.length) forumEl.innerHTML = '<p>No discussions yet.</p>';
      else forumEl.innerHTML = forum.map(f => `<div class="item"><strong>${f.title}</strong><p>by ${f.author_name}</p></div>`).join('');
    }

    document.getElementById('join').onclick = async () => {
      const pin = document.getElementById('pin').value.trim();
      const name = document.getElementById('name').value.trim();
      const err = document.getElementById('err');
      err.classList.add('hidden');
      try {
        const res = await fetch('/api/hub/join', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ pin, name })
        });
        const data = await res.json();
        if (!res.ok) throw new Error('Invalid PIN or name');
        studentName = name;
        localStorage.setItem('classmate_student_name', name);
        document.getElementById('join-panel').classList.add('hidden');
        document.getElementById('class-panel').classList.remove('hidden');
        document.getElementById('welcome').textContent = data.course_title;
        document.getElementById('welcome-msg').textContent = data.message;
        await loadClassContent();
      } catch (e) {
        err.textContent = e.message || 'Could not join. Check PIN and try again.';
        err.classList.remove('hidden');
      }
    };
  </script>
</body>
</html>"##;
