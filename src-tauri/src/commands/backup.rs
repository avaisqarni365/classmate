use crate::commands::sync::{apply_backup, build_backup};
use crate::models::{AutoBackupEntry, AutoBackupSettings, AutoBackupStatus, BackupPayload, SyncPeerResult};
use crate::AppState;
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

fn read_setting(conn: &Connection, key: &str, default: &str) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .unwrap_or_else(|_| default.to_string())
}

fn write_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn backup_dir(app_data: &Path) -> PathBuf {
    app_data.join("backups")
}

fn read_settings(conn: &Connection) -> AutoBackupSettings {
    let enabled = read_setting(conn, "auto_backup_enabled", "false") == "true";
    let interval = read_setting(conn, "auto_backup_interval", "daily");
    let max_keep: i64 = read_setting(conn, "auto_backup_max_keep", "7")
        .parse()
        .unwrap_or(7)
        .clamp(1, 30);
    AutoBackupSettings {
        enabled,
        interval,
        max_keep,
    }
}

fn parse_last_at(conn: &Connection) -> Option<DateTime<Utc>> {
    let raw = read_setting(conn, "auto_backup_last_at", "");
    if raw.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(&raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn is_due(settings: &AutoBackupSettings, last_at: Option<DateTime<Utc>>) -> bool {
    if !settings.enabled || settings.interval == "off" {
        return false;
    }
    let Some(last) = last_at else {
        return true;
    };
    let elapsed = Utc::now() - last;
    match settings.interval.as_str() {
        "weekly" => elapsed >= Duration::days(7),
        _ => elapsed >= Duration::days(1),
    }
}

fn next_due_at(settings: &AutoBackupSettings, last_at: Option<DateTime<Utc>>) -> Option<String> {
    if !settings.enabled || settings.interval == "off" {
        return None;
    }
    let last = last_at.unwrap_or_else(Utc::now);
    let next = match settings.interval.as_str() {
        "weekly" => last + Duration::days(7),
        _ => last + Duration::days(1),
    };
    Some(next.to_rfc3339())
}

pub fn prune_backups(app_data: &Path, max_keep: i64) -> Result<(), String> {
    let dir = backup_dir(app_data);
    if !dir.exists() {
        return Ok(());
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    files.sort_by(|a, b| b.cmp(a));
    for path in files.into_iter().skip(max_keep as usize) {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

pub fn write_backup_file(app_data: &Path, conn: &Connection) -> Result<AutoBackupEntry, String> {
    let dir = backup_dir(app_data);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let payload = build_backup(conn)?;
    let filename = format!(
        "classmate-backup-{}.json",
        payload.exported_at.replace(':', "-")
    );
    let path = dir.join(&filename);
    let json = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    let size_bytes = json.len() as u64;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    write_setting(conn, "auto_backup_last_at", &payload.exported_at)?;
    Ok(AutoBackupEntry {
        filename,
        exported_at: payload.exported_at,
        size_bytes,
    })
}

pub fn maybe_run_scheduled_backup(app_data: &Path, conn: &Connection) -> Result<Option<AutoBackupEntry>, String> {
    let settings = read_settings(conn);
    let last_at = parse_last_at(conn);
    if !is_due(&settings, last_at) {
        return Ok(None);
    }
    let entry = write_backup_file(app_data, conn)?;
    prune_backups(app_data, settings.max_keep)?;
    Ok(Some(entry))
}

fn list_backup_files(app_data: &Path) -> Result<Vec<AutoBackupEntry>, String> {
    let dir = backup_dir(app_data);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let size_bytes = entry.metadata().map_err(|e| e.to_string())?.len();
        let exported_at = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<BackupPayload>(&text).ok())
            .map(|p| p.exported_at)
            .unwrap_or_else(|| filename.clone());
        entries.push(AutoBackupEntry {
            filename,
            exported_at,
            size_bytes,
        });
    }
    entries.sort_by(|a, b| b.exported_at.cmp(&a.exported_at));
    Ok(entries)
}

#[tauri::command]
pub fn get_auto_backup_status(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<AutoBackupStatus, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let settings = read_settings(&conn);
    let last_at = parse_last_at(&conn).map(|dt| dt.to_rfc3339());
    let next_due = next_due_at(&settings, parse_last_at(&conn));
    let backups = list_backup_files(&app_data)?;
    Ok(AutoBackupStatus {
        settings,
        last_backup_at: last_at,
        next_due_at: next_due,
        backup_count: backups.len() as i64,
        backup_dir: backup_dir(&app_data).to_string_lossy().into(),
    })
}

#[tauri::command]
pub fn set_auto_backup_settings(
    state: State<'_, AppState>,
    input: AutoBackupSettings,
) -> Result<(), String> {
    if !["daily", "weekly", "off"].contains(&input.interval.as_str()) {
        return Err("Interval must be daily, weekly, or off".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    write_setting(
        &conn,
        "auto_backup_enabled",
        if input.enabled { "true" } else { "false" },
    )?;
    write_setting(&conn, "auto_backup_interval", &input.interval)?;
    write_setting(
        &conn,
        "auto_backup_max_keep",
        &input.max_keep.clamp(1, 30).to_string(),
    )?;
    Ok(())
}

#[tauri::command]
pub fn run_auto_backup_now(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<AutoBackupEntry, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let settings = read_settings(&conn);
    let entry = write_backup_file(&app_data, &conn)?;
    prune_backups(&app_data, settings.max_keep)?;
    Ok(entry)
}

#[tauri::command]
pub fn list_auto_backups(
    app: AppHandle,
) -> Result<Vec<AutoBackupEntry>, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    list_backup_files(&app_data)
}

#[tauri::command]
pub fn restore_auto_backup(
    state: State<'_, AppState>,
    app: AppHandle,
    filename: String,
) -> Result<(), String> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("Invalid backup filename".into());
    }
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = backup_dir(&app_data).join(&filename);
    if !path.exists() {
        return Err("Backup file not found".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let payload: BackupPayload = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    apply_backup(&conn, payload)
}

pub fn build_plugnmeet_url(base: &str, room: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    let room = room.trim();
    if room.is_empty() {
        format!("{base}/")
    } else {
        format!("{base}/?roomId={room}")
    }
}

pub fn resolve_hub_video_url(
    conn: &Connection,
    app_data: &Path,
    video: &crate::video::VideoRuntime,
) -> Option<String> {
    if let Ok(url) = video.start(app_data) {
        return Some(url);
    }
    let plugnmeet_base = read_setting(conn, "plugnmeet_base_url", "");
    if !plugnmeet_base.trim().is_empty() {
        let room = read_setting(conn, "plugnmeet_room", "classmate");
        return Some(build_plugnmeet_url(&plugnmeet_base, &room));
    }
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'external_video_url'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .filter(|s| !s.trim().is_empty())
}

#[tauri::command]
pub fn push_backup_to_cloud(state: State<'_, AppState>) -> Result<SyncPeerResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let url = read_setting(&conn, "cloud_backup_url", "");
    let token = read_setting(&conn, "cloud_backup_token", "");
    if url.trim().is_empty() {
        return Err("Cloud backup URL is not configured".into());
    }
    if token.trim().len() < 6 {
        return Err("Cloud backup token must be at least 6 characters".into());
    }
    let payload = build_backup(&conn)?;
    let exported_at = payload.exported_at.clone();
    let base = url.trim().trim_end_matches('/');
    let target = if base.ends_with("/api/sync/backup") {
        base.to_string()
    } else {
        format!("{base}/api/sync/backup")
    };
    ureq::post(&target)
        .set("x-sync-token", &token)
        .send_json(payload)
        .map_err(|e| format!("Cloud push failed: {e}"))?;
    Ok(SyncPeerResult {
        message: "Backup pushed to cloud endpoint".into(),
        exported_at,
    })
}
