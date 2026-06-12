use crate::models::{BackupPayload, ImportResult, SettingEntry, SyncPeerResult, SyncServerStatus, UiPreferences};
use crate::AppState;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

fn table_to_json(conn: &rusqlite::Connection, table: &str) -> Result<Vec<serde_json::Value>, String> {
    let sql = format!("SELECT * FROM {table}");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let mut obj = serde_json::Map::new();
            for (i, name) in col_names.iter().enumerate() {
                let val: rusqlite::types::Value = row.get(i)?;
                obj.insert(name.clone(), sqlite_value_to_json(val));
            }
            Ok(serde_json::Value::Object(obj))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

fn sqlite_value_to_json(value: rusqlite::types::Value) -> serde_json::Value {
    match value {
        rusqlite::types::Value::Null => serde_json::Value::Null,
        rusqlite::types::Value::Integer(i) => serde_json::Value::from(i),
        rusqlite::types::Value::Real(f) => serde_json::Value::from(f),
        rusqlite::types::Value::Text(s) => serde_json::Value::from(s),
        rusqlite::types::Value::Blob(b) => {
            serde_json::Value::from(base64_encode(&b))
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 63) as usize] as char);
        out.push(TABLE[((triple >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(triple & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[tauri::command]
pub fn export_backup(state: State<'_, AppState>) -> Result<BackupPayload, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    build_backup(&conn)
}

pub fn build_backup(conn: &rusqlite::Connection) -> Result<BackupPayload, String> {
    Ok(BackupPayload {
        version: "2.0".into(),
        exported_at: Utc::now().to_rfc3339(),
        users: table_to_json(conn, "users")?,
        courses: table_to_json(conn, "courses")?,
        enrollments: table_to_json(conn, "enrollments")?,
        assignments: table_to_json(conn, "assignments")?,
        grades: table_to_json(conn, "grades")?,
        materials: table_to_json(conn, "course_materials")?,
        announcements: table_to_json(conn, "announcements")?,
        settings: table_to_json(conn, "settings")?,
        schools: table_to_json(conn, "schools")?,
        school_members: table_to_json(conn, "school_members")?,
        class_sessions: table_to_json(conn, "class_sessions")?,
        attendance: table_to_json(conn, "attendance")?,
        forum_topics: table_to_json(conn, "forum_topics")?,
        forum_posts: table_to_json(conn, "forum_posts")?,
        parent_links: table_to_json(conn, "parent_links")?,
        lti_tools: table_to_json(conn, "lti_tools")?,
        certificates: table_to_json(conn, "certificates")?,
        quizzes: table_to_json(conn, "quizzes")?,
        quiz_questions: table_to_json(conn, "quiz_questions")?,
        quiz_attempts: table_to_json(conn, "quiz_attempts")?,
        schedule_slots: table_to_json(conn, "schedule_slots")?,
        session_polls: table_to_json(conn, "session_polls")?,
        session_poll_votes: table_to_json(conn, "session_poll_votes")?,
        assignment_submissions: table_to_json(conn, "assignment_submissions")?,
        assignment_rubrics: table_to_json(conn, "assignment_rubrics")?,
        whatsapp_groups: table_to_json(conn, "whatsapp_groups")?,
        whatsapp_group_members: table_to_json(conn, "whatsapp_group_members")?,
        whatsapp_group_links: table_to_json(conn, "whatsapp_group_links")?,
        whatsapp_consent: table_to_json(conn, "whatsapp_consent")?,
        whatsapp_outbound_messages: table_to_json(conn, "whatsapp_outbound_messages")?,
        whatsapp_inbound_messages: table_to_json(conn, "whatsapp_inbound_messages")?,
        whatsapp_scheduled_broadcasts: table_to_json(conn, "whatsapp_scheduled_broadcasts")?,
        whatsapp_consent_log: table_to_json(conn, "whatsapp_consent_log")?,
        email_log: table_to_json(conn, "email_log")?,
        push_devices: table_to_json(conn, "push_devices")?,
        push_log: table_to_json(conn, "push_log")?,
        whatsapp_message_status_events: table_to_json(conn, "whatsapp_message_status_events")?,
    })
}

#[tauri::command]
pub fn import_backup(state: State<'_, AppState>, payload: BackupPayload) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    apply_backup(&conn, payload)
}

pub fn apply_backup(conn: &rusqlite::Connection, payload: BackupPayload) -> Result<(), String> {
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
    let result = if payload.version.starts_with('2') {
        import_v2(conn, &payload)
    } else {
        import_v1(conn, &payload)
    };
    if result.is_ok() {
        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
    } else {
        let _ = conn.execute_batch("ROLLBACK");
    }
    result
}

fn import_v1(conn: &rusqlite::Connection, payload: &BackupPayload) -> Result<(), String> {
    for table in [
        "grades",
        "assignments",
        "enrollments",
        "course_materials",
        "announcements",
        "courses",
        "users",
        "settings",
    ] {
        conn.execute(&format!("DELETE FROM {table}"), [])
            .map_err(|e| e.to_string())?;
    }
    import_rows(conn, "users", &payload.users)?;
    import_rows(conn, "courses", &payload.courses)?;
    import_rows(conn, "enrollments", &payload.enrollments)?;
    import_rows(conn, "assignments", &payload.assignments)?;
    import_rows(conn, "grades", &payload.grades)?;
    import_rows(conn, "course_materials", &payload.materials)?;
    import_rows(conn, "announcements", &payload.announcements)?;
    import_rows(conn, "settings", &payload.settings)?;
    Ok(())
}

fn import_v2(conn: &rusqlite::Connection, payload: &BackupPayload) -> Result<(), String> {
    for table in [
        "whatsapp_consent_log",
        "whatsapp_message_status_events",
        "whatsapp_scheduled_broadcasts",
        "whatsapp_inbound_messages",
        "email_log",
        "push_log",
        "push_devices",
        "whatsapp_outbound_messages",
        "whatsapp_consent",
        "whatsapp_group_links",
        "whatsapp_group_members",
        "whatsapp_groups",
        "session_poll_votes",
        "session_polls",
        "assignment_submissions",
        "assignment_rubrics",
        "quiz_attempts",
        "quiz_questions",
        "quizzes",
        "attendance",
        "class_sessions",
        "schedule_slots",
        "forum_posts",
        "forum_topics",
        "certificates",
        "parent_links",
        "grades",
        "enrollments",
        "announcements",
        "assignments",
        "course_materials",
        "lti_tools",
        "courses",
        "school_members",
        "users",
        "schools",
        "settings",
    ] {
        conn.execute(&format!("DELETE FROM {table}"), [])
            .map_err(|e| e.to_string())?;
    }
    import_rows(conn, "schools", &payload.schools)?;
    import_rows(conn, "users", &payload.users)?;
    import_rows(conn, "school_members", &payload.school_members)?;
    import_rows(conn, "courses", &payload.courses)?;
    import_rows(conn, "enrollments", &payload.enrollments)?;
    import_rows(conn, "assignments", &payload.assignments)?;
    import_rows(conn, "grades", &payload.grades)?;
    import_rows(conn, "course_materials", &payload.materials)?;
    import_rows(conn, "announcements", &payload.announcements)?;
    import_rows(conn, "forum_topics", &payload.forum_topics)?;
    import_rows(conn, "forum_posts", &payload.forum_posts)?;
    import_rows(conn, "parent_links", &payload.parent_links)?;
    import_rows(conn, "lti_tools", &payload.lti_tools)?;
    import_rows(conn, "certificates", &payload.certificates)?;
    import_rows(conn, "class_sessions", &payload.class_sessions)?;
    import_rows(conn, "attendance", &payload.attendance)?;
    import_rows(conn, "schedule_slots", &payload.schedule_slots)?;
    import_rows(conn, "quizzes", &payload.quizzes)?;
    import_rows(conn, "quiz_questions", &payload.quiz_questions)?;
    import_rows(conn, "quiz_attempts", &payload.quiz_attempts)?;
    import_rows(conn, "session_polls", &payload.session_polls)?;
    import_rows(conn, "session_poll_votes", &payload.session_poll_votes)?;
    import_rows(conn, "assignment_submissions", &payload.assignment_submissions)?;
    import_rows(conn, "assignment_rubrics", &payload.assignment_rubrics)?;
    import_rows(conn, "whatsapp_groups", &payload.whatsapp_groups)?;
    import_rows(conn, "whatsapp_group_members", &payload.whatsapp_group_members)?;
    import_rows(conn, "whatsapp_group_links", &payload.whatsapp_group_links)?;
    import_rows(conn, "whatsapp_consent", &payload.whatsapp_consent)?;
    import_rows(
        conn,
        "whatsapp_outbound_messages",
        &payload.whatsapp_outbound_messages,
    )?;
    import_rows(
        conn,
        "whatsapp_inbound_messages",
        &payload.whatsapp_inbound_messages,
    )?;
    import_rows(
        conn,
        "whatsapp_scheduled_broadcasts",
        &payload.whatsapp_scheduled_broadcasts,
    )?;
    import_rows(conn, "whatsapp_consent_log", &payload.whatsapp_consent_log)?;
    import_rows(conn, "email_log", &payload.email_log)?;
    import_rows(conn, "push_devices", &payload.push_devices)?;
    import_rows(conn, "push_log", &payload.push_log)?;
    import_rows(conn, "settings", &payload.settings)?;
    Ok(())
}

fn import_rows(
    conn: &rusqlite::Connection,
    table: &str,
    rows: &[serde_json::Value],
) -> Result<(), String> {
    for row in rows {
        let obj = row.as_object().ok_or("Invalid row")?;
        if obj.is_empty() {
            continue;
        }
        let cols: Vec<&str> = obj.keys().map(String::as_str).collect();
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "INSERT INTO {table} ({}) VALUES ({})",
            cols.join(", "),
            placeholders.join(", ")
        );
        let values: Vec<rusqlite::types::Value> = cols
            .iter()
            .map(|c| json_to_sqlite(obj.get(*c).unwrap_or(&serde_json::Value::Null)))
            .collect();
        conn.execute(&sql, rusqlite::params_from_iter(values.iter()))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn json_to_sqlite(value: &serde_json::Value) -> rusqlite::types::Value {
    match value {
        serde_json::Value::Null => rusqlite::types::Value::Null,
        serde_json::Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else {
                rusqlite::types::Value::Real(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => rusqlite::types::Value::Text(s.clone()),
        _ => rusqlite::types::Value::Text(value.to_string()),
    }
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let val: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .ok();
    Ok(val)
}

#[tauri::command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting_work(&conn, &key, &value)
}

pub fn set_setting_work(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_settings(state: State<'_, AppState>) -> Result<Vec<SettingEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings ORDER BY key")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SettingEntry {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn read_pref(conn: &rusqlite::Connection, key: &str, default: &str) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .unwrap_or_else(|_| default.to_string())
}

#[tauri::command]
pub fn get_ui_preferences(state: State<'_, AppState>) -> Result<UiPreferences, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(UiPreferences {
        school_name: read_pref(&conn, "school_name", "ClassMate"),
        theme: read_pref(&conn, "theme", "default"),
        font_scale: read_pref(&conn, "font_scale", "100"),
        accent_color: read_pref(&conn, "accent_color", "#2563eb"),
        locale: read_pref(&conn, "locale", "en"),
    })
}

#[tauri::command]
pub fn set_ui_preferences(
    state: State<'_, AppState>,
    input: UiPreferences,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    for (key, value) in [
        ("school_name", input.school_name),
        ("theme", input.theme),
        ("font_scale", input.font_scale),
        ("accent_color", input.accent_color),
        ("locale", input.locale),
    ] {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn import_oneroster_csv(state: State<'_, AppState>, csv: String) -> Result<ImportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut imported = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();
    let now = Utc::now().to_rfc3339();

    let mut lines = csv.lines();
    let header = lines.next().ok_or("Empty CSV")?;
    let cols: Vec<&str> = header.split(',').map(|s| s.trim()).collect();

    let idx = |name: &str| cols.iter().position(|c| c.eq_ignore_ascii_case(name));

    let email_i = idx("email").or_else(|| idx("username"));
    let given_i = idx("givenName").or_else(|| idx("given_name"));
    let family_i = idx("familyName").or_else(|| idx("family_name"));
    let role_i = idx("role");

    for (line_no, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let get = |i: Option<usize>| i.and_then(|n| parts.get(n)).map(|s| s.to_string());

        let email = get(email_i).unwrap_or_default();
        let given = get(given_i).unwrap_or_else(|| "Student".into());
        let family = get(family_i).unwrap_or_default();
        let role_raw = get(role_i).unwrap_or_else(|| "student".into());
        let role = match role_raw.to_lowercase().as_str() {
            "administrator" | "admin" => "admin",
            "teacher" => "teacher",
            "parent" | "guardian" => "parent",
            _ => "student",
        };

        if email.is_empty() {
            errors.push(format!("Line {}: missing email", line_no + 2));
            skipped += 1;
            continue;
        }

        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM users WHERE email = ?1",
                params![email],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if exists {
            skipped += 1;
            continue;
        }

        let name = format!("{given} {family}").trim().to_string();
        let password_hash = hash("changeme123", DEFAULT_COST).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();

        if let Err(e) = conn.execute(
            "INSERT INTO users (id, email, name, role, password_hash, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, email, name, role, password_hash, now, now],
        ) {
            errors.push(format!("Line {}: {e}", line_no + 2));
            skipped += 1;
        } else {
            imported += 1;
        }
    }

    Ok(ImportResult {
        imported,
        skipped,
        errors,
    })
}

pub const SETTING_PUBLIC_BASE_URL: &str = "public_base_url";
pub const SETTING_PUBLIC_HUB_PATH: &str = "public_hub_path";

fn read_setting(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .ok()
    .filter(|s: &String| !s.trim().is_empty())
}

pub fn read_public_base_url(conn: &rusqlite::Connection) -> Option<String> {
    read_setting(conn, SETTING_PUBLIC_BASE_URL)
        .map(|s| s.trim().trim_end_matches('/').to_string())
}

pub fn read_public_hub_path(conn: &rusqlite::Connection) -> String {
    read_setting(conn, SETTING_PUBLIC_HUB_PATH).unwrap_or_else(|| "/hub".to_string())
}

pub fn resolve_sync_url(conn: &rusqlite::Connection, lan_url: Option<String>) -> Option<String> {
    read_public_base_url(conn).or(lan_url)
}

pub fn resolve_webhook_url(
    conn: &rusqlite::Connection,
    lan_sync_url: Option<String>,
) -> Option<String> {
    resolve_sync_url(conn, lan_sync_url).map(|url| format!("{url}/api/whatsapp/webhook"))
}

pub fn resolve_hub_join_url(
    conn: &rusqlite::Connection,
    lan_join_url: Option<String>,
) -> Option<String> {
    if let Some(base) = read_public_base_url(conn) {
        let path = read_public_hub_path(conn).trim_end_matches('/').to_string();
        return Some(format!("{base}{path}/student"));
    }
    lan_join_url
}

fn enrich_sync_status(conn: &rusqlite::Connection, status: &mut SyncServerStatus) {
    let lan_url = status.sync_url.clone();
    status.public_base_url = read_public_base_url(conn);
    status.sync_url = resolve_sync_url(conn, lan_url.clone());
    status.webhook_url = resolve_webhook_url(conn, lan_url);
    status.hub_join_url = resolve_hub_join_url(conn, None);
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
    let token = Uuid::new_v4().to_string().replace('-', "")[..12].to_string();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('sync_token', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![token],
    )
    .map_err(|e| e.to_string())?;
    Ok(token)
}

#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<SyncServerStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let token = ensure_sync_token(&conn)?;
    let mut sync = state.sync.lock().map_err(|e| e.to_string())?;
    sync.set_token(token.clone());
    let mut status = sync.status();
    status.sync_token = token;
    enrich_sync_status(&conn, &mut status);
    Ok(status)
}

#[tauri::command]
pub fn start_sync_server(state: State<'_, AppState>) -> Result<SyncServerStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let token = ensure_sync_token(&conn)?;
    let db = state.db.clone();
    let mut sync = state.sync.lock().map_err(|e| e.to_string())?;
    sync.start(db, token.clone())?;
    let mut status = sync.status();
    status.sync_token = token;
    enrich_sync_status(&conn, &mut status);
    Ok(status)
}

#[tauri::command]
pub fn stop_sync_server(state: State<'_, AppState>) -> Result<SyncServerStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let token = ensure_sync_token(&conn)?;
    let mut sync = state.sync.lock().map_err(|e| e.to_string())?;
    sync.stop();
    let mut status = sync.status();
    status.sync_token = token;
    enrich_sync_status(&conn, &mut status);
    Ok(status)
}

#[tauri::command]
pub fn set_sync_token(state: State<'_, AppState>, token: String) -> Result<(), String> {
    if token.trim().len() < 6 {
        return Err("Sync token must be at least 6 characters".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('sync_token', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![token.trim()],
    )
    .map_err(|e| e.to_string())?;
    state
        .sync
        .lock()
        .map_err(|e| e.to_string())?
        .set_token(token.trim().to_string());
    Ok(())
}

#[tauri::command]
pub fn pull_from_peer(state: State<'_, AppState>, peer_url: String) -> Result<SyncPeerResult, String> {
    let token = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        ensure_sync_token(&conn)?
    };
    let base = peer_url.trim().trim_end_matches('/');
    let url = format!("{base}/api/sync/backup");
    let response = ureq::get(&url)
        .set("x-sync-token", &token)
        .call()
        .map_err(|e| format!("Pull failed: {e}"))?;
    let payload: BackupPayload = response
        .into_json()
        .map_err(|e| format!("Invalid backup JSON: {e}"))?;
    let exported_at = payload.exported_at.clone();
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        apply_backup(&conn, payload)?;
    }
    Ok(SyncPeerResult {
        message: "Pulled full backup from peer".into(),
        exported_at,
    })
}

#[tauri::command]
pub fn push_to_peer(state: State<'_, AppState>, peer_url: String) -> Result<SyncPeerResult, String> {
    let (token, payload) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let token = ensure_sync_token(&conn)?;
        let payload = build_backup(&conn)?;
        (token, payload)
    };
    let exported_at = payload.exported_at.clone();
    let base = peer_url.trim().trim_end_matches('/');
    let url = format!("{base}/api/sync/backup");
    ureq::post(&url)
        .set("x-sync-token", &token)
        .send_json(&payload)
        .map_err(|e| format!("Push failed: {e}"))?;
    Ok(SyncPeerResult {
        message: "Pushed full backup to peer".into(),
        exported_at,
    })
}
