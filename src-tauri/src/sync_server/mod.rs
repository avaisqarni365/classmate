use crate::commands::help::find_windows_installer;
use crate::commands::sync::{apply_backup, build_backup};
use crate::commands::whatsapp_api::{apply_webhook_payload, verify_webhook_token};
use crate::models::{BackupPayload, SyncServerStatus};
use crate::web_portal::HttpServerState;
use axum::{
    extract::{Query, State as AxumState},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tower_http::cors::CorsLayer;

const SYNC_PORT: u16 = 8766;
const TOKEN_HEADER: &str = "x-sync-token";

pub struct SyncRuntime {
    running: bool,
    sync_token: String,
    shutdown: Option<oneshot::Sender<()>>,
}

impl SyncRuntime {
    pub fn new() -> Self {
        Self {
            running: false,
            sync_token: String::new(),
            shutdown: None,
        }
    }

    pub fn status(&self) -> SyncServerStatus {
        let local_ip = local_ip_address::local_ip().ok().map(|ip| ip.to_string());
        let sync_url = local_ip.as_ref().map(|ip| format!("http://{ip}:{SYNC_PORT}"));
        SyncServerStatus {
            running: self.running,
            port: SYNC_PORT,
            local_ip,
            sync_url,
            sync_token: self.sync_token.clone(),
            public_base_url: None,
            webhook_url: None,
            hub_join_url: None,
        }
    }

    pub fn start(
        &mut self,
        db: Arc<Mutex<Connection>>,
        sync_token: String,
    ) -> Result<(), String> {
        if self.running {
            self.stop();
        }
        self.sync_token = sync_token.clone();
        let (tx, rx) = oneshot::channel();
        let token_for_routes = sync_token;

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                let app_state = HttpServerState::new(db, token_for_routes);
                let app = Router::new()
                    .merge(crate::web_portal::site_routes())
                    .route("/download", get(download_page))
                    .route("/download/win", get(download_installer))
                    .route("/help", get(help_page))
                    .route("/api/sync/info", get(sync_info))
                    .route("/api/sync/backup", get(sync_export).post(sync_import))
                    .route("/api/whatsapp/webhook", get(whatsapp_verify).post(whatsapp_event))
                    .with_state(app_state)
                    .layer(CorsLayer::permissive());

                let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{SYNC_PORT}"))
                    .await
                    .expect("Failed to bind sync port");

                axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = rx.await;
                    })
                    .await
                    .ok();
            });
        });

        self.running = true;
        self.shutdown = Some(tx);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        self.running = false;
    }

    pub fn set_token(&mut self, token: String) {
        self.sync_token = token;
    }
}

#[derive(Serialize)]
struct SyncInfoResponse {
    name: String,
    version: String,
}

#[derive(Deserialize)]
struct WhatsAppVerifyQuery {
    #[serde(rename = "hub.mode")]
    hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    hub_challenge: Option<String>,
}

fn authorize(headers: &HeaderMap, expected: &str) -> Result<(), StatusCode> {
    let token = headers
        .get(TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if token.is_empty() || token != expected {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

async fn sync_info(AxumState(state): AxumState<HttpServerState>) -> Json<SyncInfoResponse> {
    let name = {
        let conn = state.db.lock().ok();
        conn.and_then(|c| {
            c.query_row(
                "SELECT value FROM settings WHERE key = 'school_name'",
                [],
                |row| row.get(0),
            )
            .ok()
        })
        .unwrap_or_else(|| "ClassMate".into())
    };
    Json(SyncInfoResponse {
        name,
        version: "2.0".into(),
    })
}

async fn sync_export(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
) -> Result<Json<BackupPayload>, StatusCode> {
    authorize(&headers, &state.sync_token)?;
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    build_backup(&conn)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn sync_import(
    AxumState(state): AxumState<HttpServerState>,
    headers: HeaderMap,
    Json(payload): Json<BackupPayload>,
) -> Result<StatusCode, StatusCode> {
    authorize(&headers, &state.sync_token)?;
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    apply_backup(&conn, payload).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(StatusCode::OK)
}

async fn whatsapp_verify(
    AxumState(state): AxumState<HttpServerState>,
    Query(query): Query<WhatsAppVerifyQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let mode = query.hub_mode.as_deref().unwrap_or("");
    let token = query.hub_verify_token.as_deref().unwrap_or("");
    let challenge = query.hub_challenge.as_deref().unwrap_or("");
    if mode != "subscribe" {
        return Err(StatusCode::BAD_REQUEST);
    }
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if verify_webhook_token(&conn, token) {
        Ok(challenge.to_string())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn whatsapp_event(
    AxumState(state): AxumState<HttpServerState>,
    body: String,
) -> Result<StatusCode, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    apply_webhook_payload(&conn, &body).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(StatusCode::OK)
}

async fn download_page() -> Html<&'static str> {
    Html(include_str!("../../public_pages/download.html"))
}

async fn help_page() -> Html<&'static str> {
    Html(include_str!("../../public_pages/help.html"))
}

async fn download_installer() -> Result<impl IntoResponse, StatusCode> {
    let path = find_windows_installer().ok_or(StatusCode::NOT_FOUND)?;
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("ClassMate-setup.exe")
        .to_string();
    let bytes = tokio::task::spawn_blocking(move || std::fs::read(path))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/octet-stream"),
    );
    let disposition = format!("attachment; filename=\"{filename}\"");
    if let Ok(value) = axum::http::HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok((headers, bytes))
}
