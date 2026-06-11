use crate::AppState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_video_status(state: State<'_, AppState>, app: AppHandle) -> Result<crate::models::VideoStatus, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    Ok(state.video.status(&app_data))
}

#[tauri::command]
pub fn start_video(state: State<'_, AppState>, app: AppHandle) -> Result<crate::models::VideoStatus, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    state.video.start(&app_data)?;
    Ok(state.video.status(&app_data))
}

#[tauri::command]
pub fn stop_video(state: State<'_, AppState>, app: AppHandle) -> Result<crate::models::VideoStatus, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    state.video.stop();
    Ok(state.video.status(&app_data))
}
