use crate::db;
use crate::sync_server::SyncRuntime;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn data_dir() -> PathBuf {
    env::var("CLASSMATE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/lib/classmate"))
}

fn ensure_sync_token(conn: &rusqlite::Connection) -> Result<String, String> {
    if let Ok(token) = conn.query_row(
        "SELECT value FROM settings WHERE key = 'sync_token'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        if !token.is_empty() {
            return Ok(token);
        }
    }
    let token = uuid::Uuid::new_v4().to_string().replace('-', "")[..12].to_string();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('sync_token', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![token],
    )
    .map_err(|e| e.to_string())?;
    Ok(token)
}

pub fn run() -> Result<(), String> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let db_path = dir.join("classmate.db");
    eprintln!("ClassMate server starting (data: {})", dir.display());

    let conn = db::init(&db_path).map_err(|e| e.to_string())?;
    let db = Arc::new(Mutex::new(conn));
    let token = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        ensure_sync_token(&conn)?
    };

    let mut sync = SyncRuntime::new();
    sync.start(db, token)
        .map_err(|e| format!("failed to start sync server: {e}"))?;

    eprintln!("Sync + WhatsApp webhook listening on 0.0.0.0:8766");
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}
