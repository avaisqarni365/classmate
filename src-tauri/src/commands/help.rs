use crate::commands::sync::{read_public_base_url, read_public_hub_path, resolve_hub_join_url, resolve_webhook_url};
use crate::models::HelpInfo;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

const GITHUB_URL: &str = "https://github.com/avaisqarni365/classmate";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn download_dir() -> PathBuf {
    std::env::var("CLASSMATE_DOWNLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/lib/classmate/downloads"))
}

pub fn find_windows_installer() -> Option<PathBuf> {
    let dir = download_dir();
    if !dir.is_dir() {
        return None;
    }
    let mut matches: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
        })
        .collect();
    matches.sort();
    matches.pop()
}

#[tauri::command]
pub fn get_help_info(state: State<'_, AppState>) -> Result<HelpInfo, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sync = state.sync.lock().map_err(|e| e.to_string())?;
    let lan_sync_url = sync.status().sync_url;
    let public_base_url = read_public_base_url(&conn);
    let hub_path = read_public_hub_path(&conn);
    let webhook_url = resolve_webhook_url(&conn, lan_sync_url);
    let hub_join_url = resolve_hub_join_url(&conn, None);
    let download_web_url = public_base_url
        .as_ref()
        .map(|base| format!("{base}/download"));

    Ok(HelpInfo {
        app_version: APP_VERSION.to_string(),
        public_base_url,
        public_hub_path: Some(hub_path),
        webhook_url,
        hub_join_url,
        download_web_url,
        sync_running: sync.status().running,
        github_url: GITHUB_URL.to_string(),
        windows_installer_available: find_windows_installer().is_some(),
    })
}
