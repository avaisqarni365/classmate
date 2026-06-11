use crate::commands::tenancy::{active_school_id_or_resolve, require_user};
use crate::models::{
    PushDevice, PushLogEntry, PushReminderRunResult, PushReminderStatus, PushSettings,
    RegisterPushDeviceInput, SavePushSettingsInput, SendPushInput, SendPushResult, TestPushInput,
};
use crate::AppState;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rusqlite::{params, Connection};
use serde_json::json;
use tauri::State;
use uuid::Uuid;

const SETTING_PUSH_ENABLED: &str = "push_enabled";
const SETTING_FCM_PROJECT_ID: &str = "fcm_project_id";
const SETTING_FCM_SERVICE_ACCOUNT: &str = "fcm_service_account_json";
const SETTING_APNS_KEY_ID: &str = "apns_key_id";
const SETTING_APNS_TEAM_ID: &str = "apns_team_id";
const SETTING_APNS_BUNDLE_ID: &str = "apns_bundle_id";
const SETTING_APNS_PRIVATE_KEY: &str = "apns_private_key";
const SETTING_APNS_SANDBOX: &str = "apns_use_sandbox";
const SETTING_PUSH_REMINDERS_ENABLED: &str = "push_reminders_enabled";
const SETTING_PUSH_REMINDERS_LAST_AT: &str = "push_reminders_last_at";

fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .ok()
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

struct FcmConfig {
    project_id: String,
    service_account_json: String,
}

struct ApnsConfig {
    key_id: String,
    team_id: String,
    bundle_id: String,
    private_key: String,
    use_sandbox: bool,
}

pub fn push_is_enabled(conn: &Connection) -> bool {
    get_setting(conn, SETTING_PUSH_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false)
}

fn fcm_is_configured(conn: &Connection) -> bool {
    get_setting(conn, SETTING_FCM_SERVICE_ACCOUNT)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        && get_setting(conn, SETTING_FCM_PROJECT_ID)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
}

fn apns_is_configured(conn: &Connection) -> bool {
    get_setting(conn, SETTING_APNS_PRIVATE_KEY)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        && get_setting(conn, SETTING_APNS_KEY_ID)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        && get_setting(conn, SETTING_APNS_TEAM_ID)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        && get_setting(conn, SETTING_APNS_BUNDLE_ID)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
}

fn load_fcm_config(conn: &Connection) -> Result<FcmConfig, String> {
    let project_id = get_setting(conn, SETTING_FCM_PROJECT_ID)
        .filter(|v| !v.trim().is_empty())
        .ok_or("FCM project ID is not configured")?;
    let service_account_json = get_setting(conn, SETTING_FCM_SERVICE_ACCOUNT)
        .filter(|v| !v.trim().is_empty())
        .ok_or("FCM service account JSON is not configured")?;
    Ok(FcmConfig {
        project_id,
        service_account_json,
    })
}

fn load_apns_config(conn: &Connection) -> Result<ApnsConfig, String> {
    Ok(ApnsConfig {
        key_id: get_setting(conn, SETTING_APNS_KEY_ID)
            .filter(|v| !v.trim().is_empty())
            .ok_or("APNs key ID is not configured")?,
        team_id: get_setting(conn, SETTING_APNS_TEAM_ID)
            .filter(|v| !v.trim().is_empty())
            .ok_or("APNs team ID is not configured")?,
        bundle_id: get_setting(conn, SETTING_APNS_BUNDLE_ID)
            .filter(|v| !v.trim().is_empty())
            .ok_or("APNs bundle ID is not configured")?,
        private_key: get_setting(conn, SETTING_APNS_PRIVATE_KEY)
            .filter(|v| !v.trim().is_empty())
            .ok_or("APNs private key is not configured")?,
        use_sandbox: get_setting(conn, SETTING_APNS_SANDBOX)
            .map(|v| v == "true")
            .unwrap_or(false),
    })
}

fn fcm_access_token(service_account_json: &str) -> Result<String, String> {
    let sa: serde_json::Value =
        serde_json::from_str(service_account_json).map_err(|e| format!("Invalid FCM JSON: {e}"))?;
    let client_email = sa["client_email"]
        .as_str()
        .ok_or("FCM JSON missing client_email")?;
    let private_key = sa["private_key"]
        .as_str()
        .ok_or("FCM JSON missing private_key")?;

    let now = Utc::now().timestamp();
    let claims = json!({
        "iss": client_email,
        "scope": "https://www.googleapis.com/auth/firebase.messaging",
        "aud": "https://oauth2.googleapis.com/token",
        "iat": now,
        "exp": now + 3600
    });

    let key = EncodingKey::from_rsa_pem(private_key.as_bytes())
        .map_err(|e| format!("Invalid FCM private key: {e}"))?;
    let jwt = encode(&Header::new(Algorithm::RS256), &claims, &key)
        .map_err(|e| format!("FCM JWT error: {e}"))?;

    let resp = ureq::post("https://oauth2.googleapis.com/token")
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&format!(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={jwt}"
        ))
        .map_err(|e| format!("FCM OAuth request failed: {e}"))?;

    if resp.status() >= 400 {
        return Err(format!(
            "FCM OAuth failed ({}): {}",
            resp.status(),
            resp.into_string().unwrap_or_default()
        ));
    }

    let body: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    body["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "FCM OAuth response missing access_token".to_string())
}

fn apns_bearer_token(config: &ApnsConfig) -> Result<String, String> {
    let key = EncodingKey::from_ec_pem(config.private_key.as_bytes())
        .map_err(|e| format!("Invalid APNs private key: {e}"))?;
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(config.key_id.clone());
    let claims = json!({
        "iss": config.team_id,
        "iat": Utc::now().timestamp()
    });
    encode(&header, &claims, &key).map_err(|e| format!("APNs JWT error: {e}"))
}

fn send_fcm(config: &FcmConfig, token: &str, title: &str, body: &str) -> Result<String, String> {
    let access_token = fcm_access_token(&config.service_account_json)?;
    let url = format!(
        "https://fcm.googleapis.com/v1/projects/{}/messages:send",
        config.project_id
    );
    let payload = json!({
        "message": {
            "token": token,
            "notification": {
                "title": title,
                "body": body
            }
        }
    });

    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {access_token}"))
        .set("Content-Type", "application/json")
        .send_json(payload)
        .map_err(|e| format!("FCM send failed: {e}"))?;

    let status = resp.status();
    let text = resp.into_string().unwrap_or_default();
    if status >= 400 {
        return Err(format!("FCM error ({status}): {text}"));
    }
    Ok(text)
}

fn send_apns(config: &ApnsConfig, token: &str, title: &str, body: &str) -> Result<String, String> {
    let bearer = apns_bearer_token(config)?;
    let host = if config.use_sandbox {
        "api.sandbox.push.apple.com"
    } else {
        "api.push.apple.com"
    };
    let url = format!("https://{host}/3/device/{token}");
    let payload = json!({
        "aps": {
            "alert": {
                "title": title,
                "body": body
            },
            "sound": "default"
        }
    });

    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(&url)
        .header("authorization", format!("bearer {bearer}"))
        .header("apns-topic", &config.bundle_id)
        .header("apns-push-type", "alert")
        .header("apns-priority", "10")
        .json(&payload)
        .send()
        .map_err(|e| format!("APNs send failed: {e}"))?;

    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    if !status.is_success() {
        return Err(format!("APNs error ({status}): {text}"));
    }
    Ok(if text.is_empty() {
        "ok".into()
    } else {
        text
    })
}

fn log_push(
    conn: &Connection,
    user_id: Option<&str>,
    platform: &str,
    token: &str,
    title: &str,
    body: &str,
    status: &str,
    provider_response: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO push_log (id, user_id, platform, token, title, body, status, provider_response, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            Uuid::new_v4().to_string(),
            user_id,
            platform,
            token,
            title,
            body,
            status,
            provider_response,
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn send_push_to_device(
    conn: &Connection,
    user_id: Option<&str>,
    platform: &str,
    token: &str,
    title: &str,
    body: &str,
) -> Result<String, String> {
    if !push_is_enabled(conn) {
        return Err("Push notifications are disabled".into());
    }

    let result = match platform {
        "fcm" => {
            let config = load_fcm_config(conn)?;
            send_fcm(&config, token, title, body)
        }
        "apns" => {
            let config = load_apns_config(conn)?;
            send_apns(&config, token, title, body)
        }
        _ => Err("Invalid platform".into()),
    };

    match &result {
        Ok(response) => {
            let _ = log_push(
                conn,
                user_id,
                platform,
                token,
                title,
                body,
                "sent",
                Some(response),
            );
        }
        Err(err) => {
            let _ = log_push(
                conn,
                user_id,
                platform,
                token,
                title,
                body,
                "failed",
                Some(err),
            );
        }
    }

    result
}

fn upsert_push_device(
    conn: &Connection,
    user_id: &str,
    platform: &str,
    token: &str,
    device_name: Option<&str>,
) -> Result<PushDevice, String> {
    if platform != "fcm" && platform != "apns" {
        return Err("Platform must be fcm or apns".into());
    }
    if token.trim().len() < 8 {
        return Err("Invalid device token".into());
    }

    let now = Utc::now().to_rfc3339();
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM push_devices WHERE platform = ?1 AND token = ?2",
            params![platform, token.trim()],
            |row| row.get(0),
        )
        .ok();

    let id = existing.unwrap_or_else(|| Uuid::new_v4().to_string());
    conn.execute(
        "INSERT INTO push_devices (id, user_id, platform, token, device_name, created_at, last_seen_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(platform, token) DO UPDATE SET
           user_id = excluded.user_id,
           device_name = excluded.device_name,
           last_seen_at = excluded.last_seen_at",
        params![
            id,
            user_id,
            platform,
            token.trim(),
            device_name,
            now,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(PushDevice {
        id,
        user_id: user_id.to_string(),
        platform: platform.to_string(),
        token: token.trim().to_string(),
        device_name: device_name.map(|s| s.to_string()),
        created_at: now.clone(),
        last_seen_at: now,
    })
}

pub fn register_push_device_for_student(
    conn: &Connection,
    course_id: &str,
    student_name: &str,
    platform: &str,
    token: &str,
    device_name: Option<&str>,
) -> Result<PushDevice, String> {
    let user_id: String = conn
        .query_row(
            "SELECT u.id FROM users u
             JOIN enrollments e ON e.student_id = u.id
             WHERE e.course_id = ?1 AND e.status = 'active' AND u.role = 'student'
               AND lower(u.name) = lower(?2)
             LIMIT 1",
            params![course_id, student_name.trim()],
            |row| row.get(0),
        )
        .map_err(|_| "Student not found in this course".to_string())?;

    upsert_push_device(conn, &user_id, platform, token, device_name)
}

pub fn run_assignment_push_reminders(conn: &Connection) -> Result<PushReminderRunResult, String> {
    if !push_is_enabled(conn) {
        return Err("Push notifications are disabled".into());
    }
    if !fcm_is_configured(conn) && !apns_is_configured(conn) {
        return Err("Configure FCM or APNs before sending reminders".into());
    }

    let now = Utc::now();
    let cutoff = (now + Duration::hours(48)).to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT u.id, u.name, a.title, c.title, a.due_at
             FROM assignments a
             JOIN courses c ON c.id = a.course_id
             JOIN enrollments e ON e.course_id = c.id AND e.status = 'active'
             JOIN users u ON u.id = e.student_id
             JOIN push_devices pd ON pd.user_id = u.id
             LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = u.id
             WHERE a.due_at IS NOT NULL
               AND a.due_at <= ?1
               AND a.due_at >= ?2
               AND (g.points IS NULL)",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, String, String, String, String)> = stmt
        .query_map(params![cutoff, now.to_rfc3339()], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut sent = 0i64;
    let mut failed = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (user_id, _name, assignment_title, course_title, due_at) in rows {
        let devices = list_devices_for_user(conn, &user_id)?;
        if devices.is_empty() {
            skipped += 1;
            continue;
        }

        let title = format!("Due soon: {assignment_title}");
        let body = format!("{course_title} — due {due_at}");
        for device in devices {
            match send_push_to_device(
                conn,
                Some(&user_id),
                &device.platform,
                &device.token,
                &title,
                &body,
            ) {
                Ok(_) => sent += 1,
                Err(err) => {
                    failed += 1;
                    if errors.len() < 5 {
                        errors.push(err);
                    }
                }
            }
        }
    }

    set_setting(conn, SETTING_PUSH_REMINDERS_LAST_AT, &Utc::now().to_rfc3339())?;

    Ok(PushReminderRunResult {
        sent,
        failed,
        skipped,
        errors,
    })
}

pub fn maybe_run_scheduled_push_reminders(
    conn: &Connection,
) -> Result<Option<PushReminderRunResult>, String> {
    if !push_is_enabled(conn) {
        return Ok(None);
    }
    let enabled = get_setting(conn, SETTING_PUSH_REMINDERS_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false);
    if !enabled {
        return Ok(None);
    }

    let last_at: Option<DateTime<Utc>> = get_setting(conn, SETTING_PUSH_REMINDERS_LAST_AT)
        .and_then(|raw| DateTime::parse_from_rfc3339(&raw).ok().map(|dt| dt.with_timezone(&Utc)));

    let due = match last_at {
        None => true,
        Some(last) => Utc::now() - last >= Duration::hours(24),
    };

    if !due {
        return Ok(None);
    }

    run_assignment_push_reminders(conn).map(Some)
}

fn list_devices_for_user(conn: &Connection, user_id: &str) -> Result<Vec<PushDevice>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, user_id, platform, token, device_name, created_at, last_seen_at
             FROM push_devices WHERE user_id = ?1 ORDER BY last_seen_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let devices = stmt
        .query_map(params![user_id], map_push_device)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(devices)
}

fn map_push_device(row: &rusqlite::Row<'_>) -> rusqlite::Result<PushDevice> {
    Ok(PushDevice {
        id: row.get(0)?,
        user_id: row.get(1)?,
        platform: row.get(2)?,
        token: row.get(3)?,
        device_name: row.get(4)?,
        created_at: row.get(5)?,
        last_seen_at: row.get(6)?,
    })
}

#[tauri::command]
pub fn get_push_settings(state: State<'_, AppState>) -> Result<PushSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(read_push_settings(&conn))
}

fn read_push_settings(conn: &Connection) -> PushSettings {
    PushSettings {
        enabled: push_is_enabled(conn),
        fcm_configured: fcm_is_configured(conn),
        fcm_project_id: get_setting(conn, SETTING_FCM_PROJECT_ID).unwrap_or_default(),
        fcm_service_account_set: get_setting(conn, SETTING_FCM_SERVICE_ACCOUNT)
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        apns_configured: apns_is_configured(conn),
        apns_key_id: get_setting(conn, SETTING_APNS_KEY_ID).unwrap_or_default(),
        apns_team_id: get_setting(conn, SETTING_APNS_TEAM_ID).unwrap_or_default(),
        apns_bundle_id: get_setting(conn, SETTING_APNS_BUNDLE_ID).unwrap_or_default(),
        apns_private_key_set: get_setting(conn, SETTING_APNS_PRIVATE_KEY)
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        apns_use_sandbox: get_setting(conn, SETTING_APNS_SANDBOX)
            .map(|v| v == "true")
            .unwrap_or(false),
    }
}

#[tauri::command]
pub fn save_push_settings(
    state: State<'_, AppState>,
    input: SavePushSettingsInput,
) -> Result<PushSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        SETTING_PUSH_ENABLED,
        if input.enabled { "true" } else { "false" },
    )?;
    set_setting(&conn, SETTING_FCM_PROJECT_ID, input.fcm_project_id.trim())?;
    if let Some(json) = input.fcm_service_account_json {
        if !json.trim().is_empty() {
            set_setting(&conn, SETTING_FCM_SERVICE_ACCOUNT, json.trim())?;
        }
    }
    set_setting(&conn, SETTING_APNS_KEY_ID, input.apns_key_id.trim())?;
    set_setting(&conn, SETTING_APNS_TEAM_ID, input.apns_team_id.trim())?;
    set_setting(&conn, SETTING_APNS_BUNDLE_ID, input.apns_bundle_id.trim())?;
    if let Some(key) = input.apns_private_key {
        if !key.trim().is_empty() {
            set_setting(&conn, SETTING_APNS_PRIVATE_KEY, key.trim())?;
        }
    }
    set_setting(
        &conn,
        SETTING_APNS_SANDBOX,
        if input.apns_use_sandbox {
            "true"
        } else {
            "false"
        },
    )?;
    Ok(read_push_settings(&conn))
}

#[tauri::command]
pub fn get_push_reminder_status(state: State<'_, AppState>) -> Result<PushReminderStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let settings = read_push_settings(&conn);
    let enabled = get_setting(&conn, SETTING_PUSH_REMINDERS_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false);
    let last_run_at = get_setting(&conn, SETTING_PUSH_REMINDERS_LAST_AT);
    let device_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM push_devices", [], |row| row.get(0))
        .unwrap_or(0);
    Ok(PushReminderStatus {
        push_configured: settings.fcm_configured || settings.apns_configured,
        enabled,
        last_run_at,
        device_count,
    })
}

#[tauri::command]
pub fn set_push_reminder_settings(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        SETTING_PUSH_REMINDERS_ENABLED,
        if enabled { "true" } else { "false" },
    )
}

#[tauri::command]
pub fn run_push_reminders_now(state: State<'_, AppState>) -> Result<PushReminderRunResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    run_assignment_push_reminders(&conn)
}

#[tauri::command]
pub fn test_push_notification(
    state: State<'_, AppState>,
    input: TestPushInput,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    send_push_to_device(
        &conn,
        None,
        &input.platform,
        &input.token,
        &input.title,
        &input.body,
    )
}

#[tauri::command]
pub fn register_push_device(
    state: State<'_, AppState>,
    input: RegisterPushDeviceInput,
) -> Result<PushDevice, String> {
    let user = require_user(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    upsert_push_device(
        &conn,
        &user.id,
        &input.platform,
        &input.token,
        input.device_name.as_deref(),
    )
}

#[tauri::command]
pub fn unregister_push_device(state: State<'_, AppState>, device_id: String) -> Result<(), String> {
    let user = require_user(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM push_devices WHERE id = ?1 AND user_id = ?2",
        params![device_id, user.id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_my_push_devices(state: State<'_, AppState>) -> Result<Vec<PushDevice>, String> {
    let user = require_user(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    list_devices_for_user(&conn, &user.id)
}

#[tauri::command]
pub fn send_push_notification(
    state: State<'_, AppState>,
    input: SendPushInput,
) -> Result<SendPushResult, String> {
    require_user(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;

    let in_school: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM school_members WHERE user_id = ?1 AND school_id = ?2",
            params![input.user_id, school_id],
            |row| row.get(0),
        )
        .map(|c: i64| c > 0)
        .unwrap_or(false);
    if !in_school {
        return Err("User is not in the active school".into());
    }

    let devices = list_devices_for_user(&conn, &input.user_id)?;
    if devices.is_empty() {
        return Ok(SendPushResult {
            sent: 0,
            failed: 0,
            errors: vec!["No registered devices".into()],
        });
    }

    let title = input.title.trim();
    let body = input.body.trim();
    let mut sent = 0i64;
    let mut failed = 0i64;
    let mut errors = Vec::new();

    for device in devices {
        match send_push_to_device(
            &conn,
            Some(&input.user_id),
            &device.platform,
            &device.token,
            title,
            body,
        ) {
            Ok(_) => sent += 1,
            Err(err) => {
                failed += 1;
                if errors.len() < 5 {
                    errors.push(err);
                }
            }
        }
    }

    Ok(SendPushResult {
        sent,
        failed,
        errors,
    })
}

#[tauri::command]
pub fn list_push_log(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<PushLogEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let max = limit.unwrap_or(50).clamp(1, 200);
    let mut stmt = conn
        .prepare(
            "SELECT id, user_id, platform, token, title, body, status, provider_response, created_at
             FROM push_log ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![max], |row| {
            Ok(PushLogEntry {
                id: row.get(0)?,
                user_id: row.get(1)?,
                platform: row.get(2)?,
                token: row.get(3)?,
                title: row.get(4)?,
                body: row.get(5)?,
                status: row.get(6)?,
                provider_response: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}
