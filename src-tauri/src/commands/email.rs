use crate::commands::parent::build_parent_digest;
use crate::models::{
    DigestEmailRunResult, DigestEmailSettings, DigestEmailStatus, EmailLogEntry,
    SendParentDigestEmailInput, SendParentDigestEmailResult, SaveSmtpSettingsInput,
    SmtpConnectionTest, SmtpSettings,
};
use crate::AppState;
use chrono::{DateTime, Duration, Utc};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

const SETTING_SMTP_HOST: &str = "smtp_host";
const SETTING_SMTP_PORT: &str = "smtp_port";
const SETTING_SMTP_USERNAME: &str = "smtp_username";
const SETTING_SMTP_PASSWORD: &str = "smtp_password";
const SETTING_SMTP_FROM: &str = "smtp_from";
const SETTING_SMTP_USE_TLS: &str = "smtp_use_tls";
const SETTING_DIGEST_ENABLED: &str = "digest_email_enabled";
const SETTING_DIGEST_INTERVAL: &str = "digest_email_interval";
const SETTING_DIGEST_LAST_AT: &str = "digest_email_last_at";

struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
    use_tls: bool,
}

fn get_setting(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .ok()
}

fn set_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn load_smtp_config(conn: &rusqlite::Connection) -> Result<SmtpConfig, String> {
    let host = get_setting(conn, SETTING_SMTP_HOST)
        .filter(|v| !v.trim().is_empty())
        .ok_or("SMTP host is not configured")?;
    let port: u16 = get_setting(conn, SETTING_SMTP_PORT)
        .and_then(|v| v.parse().ok())
        .unwrap_or(587);
    let username = get_setting(conn, SETTING_SMTP_USERNAME).unwrap_or_default();
    let password = get_setting(conn, SETTING_SMTP_PASSWORD)
        .filter(|v| !v.is_empty())
        .ok_or("SMTP password is missing")?;
    let from = get_setting(conn, SETTING_SMTP_FROM)
        .filter(|v| !v.trim().is_empty())
        .ok_or("From address is required")?;
    let use_tls = get_setting(conn, SETTING_SMTP_USE_TLS)
        .map(|v| v != "0" && v.to_lowercase() != "false")
        .unwrap_or(true);
    Ok(SmtpConfig {
        host,
        port,
        username,
        password,
        from,
        use_tls,
    })
}

fn build_mailer(config: &SmtpConfig) -> Result<SmtpTransport, String> {
    if config.use_tls {
        let mut builder =
            SmtpTransport::starttls_relay(&config.host).map_err(|e| e.to_string())?;
        if !config.username.is_empty() {
            builder = builder.credentials(Credentials::new(
                config.username.clone(),
                config.password.clone(),
            ));
        }
        Ok(builder.port(config.port).build())
    } else {
        let mut builder = SmtpTransport::builder_dangerous(&config.host).port(config.port);
        if !config.username.is_empty() {
            builder = builder.credentials(Credentials::new(
                config.username.clone(),
                config.password.clone(),
            ));
        }
        Ok(builder.build())
    }
}

fn send_html_email(config: &SmtpConfig, to: &str, subject: &str, html: &str) -> Result<(), String> {
    let from = config
        .from
        .parse()
        .map_err(|e| format!("Invalid from address: {e}"))?;
    let to_addr = to
        .parse()
        .map_err(|e| format!("Invalid recipient address: {e}"))?;
    let message = Message::builder()
        .from(from)
        .to(to_addr)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html.to_string())
        .map_err(|e| e.to_string())?;
    let mailer = build_mailer(config)?;
    mailer.send(&message).map_err(|e| e.to_string())?;
    Ok(())
}

fn read_digest_settings(conn: &rusqlite::Connection) -> DigestEmailSettings {
    let enabled = get_setting(conn, SETTING_DIGEST_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false);
    let interval = get_setting(conn, SETTING_DIGEST_INTERVAL).unwrap_or_else(|| "weekly".into());
    DigestEmailSettings {
        enabled,
        interval,
    }
}

fn parse_digest_last_at(conn: &rusqlite::Connection) -> Option<DateTime<Utc>> {
    get_setting(conn, SETTING_DIGEST_LAST_AT).and_then(|raw| {
        DateTime::parse_from_rfc3339(&raw)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    })
}

fn digest_is_due(settings: &DigestEmailSettings, last_at: Option<DateTime<Utc>>) -> bool {
    if !settings.enabled || settings.interval == "off" {
        return false;
    }
    let Some(last) = last_at else {
        return true;
    };
    let elapsed = Utc::now() - last;
    match settings.interval.as_str() {
        "daily" => elapsed >= Duration::days(1),
        _ => elapsed >= Duration::days(7),
    }
}

fn digest_next_due_at(
    settings: &DigestEmailSettings,
    last_at: Option<DateTime<Utc>>,
) -> Option<String> {
    if !settings.enabled || settings.interval == "off" {
        return None;
    }
    let last = last_at.unwrap_or_else(Utc::now);
    let next = match settings.interval.as_str() {
        "daily" => last + Duration::days(1),
        _ => last + Duration::days(7),
    };
    Some(next.to_rfc3339())
}

fn smtp_is_configured(conn: &rusqlite::Connection) -> bool {
    load_smtp_config(conn).is_ok()
}

fn send_digest_to_parent(
    conn: &rusqlite::Connection,
    config: &SmtpConfig,
    parent_id: &str,
    to_email: Option<&str>,
) -> Result<String, String> {
    let parent_email = if let Some(email) = to_email.filter(|e| !e.trim().is_empty()) {
        email.trim().to_string()
    } else {
        conn.query_row(
            "SELECT email FROM users WHERE id = ?1 AND role = 'parent'",
            params![parent_id],
            |row| row.get(0),
        )
        .map_err(|_| "Parent not found".to_string())?
    };
    if parent_email.trim().is_empty() {
        return Err("Parent has no email address".into());
    }

    let digest = build_parent_digest(conn, parent_id)?;
    let subject = format!(
        "ClassMate weekly digest — {}",
        &digest.generated_at[..10.min(digest.generated_at.len())]
    );

    send_html_email(config, &parent_email, &subject, &digest.html)?;
    log_email(
        conn,
        &parent_email,
        &subject,
        "parent_digest",
        "sent",
        None,
    )?;
    Ok(parent_email)
}

pub fn run_all_parent_digests(conn: &rusqlite::Connection) -> Result<DigestEmailRunResult, String> {
    let config = load_smtp_config(conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT u.id, u.email FROM users u
             JOIN parent_links pl ON pl.parent_id = u.id
             WHERE u.role = 'parent'",
        )
        .map_err(|e| e.to_string())?;
    let parents: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut sent = 0i64;
    let mut failed = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (parent_id, email) in parents {
        if email.trim().is_empty() {
            skipped += 1;
            continue;
        }
        match send_digest_to_parent(conn, &config, &parent_id, None) {
            Ok(_) => sent += 1,
            Err(err) => {
                failed += 1;
                if errors.len() < 5 {
                    errors.push(err);
                }
            }
        }
    }

    set_setting(conn, SETTING_DIGEST_LAST_AT, &Utc::now().to_rfc3339())?;

    Ok(DigestEmailRunResult {
        sent,
        failed,
        skipped,
        errors,
    })
}

pub fn maybe_run_scheduled_digest(conn: &rusqlite::Connection) -> Result<Option<DigestEmailRunResult>, String> {
    if !smtp_is_configured(conn) {
        return Ok(None);
    }
    let settings = read_digest_settings(conn);
    let last_at = parse_digest_last_at(conn);
    if !digest_is_due(&settings, last_at) {
        return Ok(None);
    }
    run_all_parent_digests(conn).map(Some)
}

fn log_email(
    conn: &rusqlite::Connection,
    recipient: &str,
    subject: &str,
    kind: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO email_log (id, recipient, subject, kind, status, error, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            Uuid::new_v4().to_string(),
            recipient,
            subject,
            kind,
            status,
            error,
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_smtp_settings(state: State<'_, AppState>) -> Result<SmtpSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let host = get_setting(&conn, SETTING_SMTP_HOST).unwrap_or_default();
    let port: i64 = get_setting(&conn, SETTING_SMTP_PORT)
        .and_then(|v| v.parse().ok())
        .unwrap_or(587);
    let username = get_setting(&conn, SETTING_SMTP_USERNAME).unwrap_or_default();
    let from = get_setting(&conn, SETTING_SMTP_FROM).unwrap_or_default();
    let password_set = get_setting(&conn, SETTING_SMTP_PASSWORD)
        .filter(|v| !v.is_empty())
        .is_some();
    let use_tls = get_setting(&conn, SETTING_SMTP_USE_TLS)
        .map(|v| v != "0" && v.to_lowercase() != "false")
        .unwrap_or(true);
    Ok(SmtpSettings {
        configured: !host.trim().is_empty() && password_set && !from.trim().is_empty(),
        host,
        port,
        username,
        from,
        password_set,
        use_tls,
    })
}

#[tauri::command]
pub fn save_smtp_settings(
    state: State<'_, AppState>,
    input: SaveSmtpSettingsInput,
) -> Result<SmtpSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(&conn, SETTING_SMTP_HOST, input.host.trim())?;
    set_setting(&conn, SETTING_SMTP_PORT, &input.port.to_string())?;
    set_setting(&conn, SETTING_SMTP_USERNAME, input.username.trim())?;
    set_setting(&conn, SETTING_SMTP_FROM, input.from.trim())?;
    set_setting(
        &conn,
        SETTING_SMTP_USE_TLS,
        if input.use_tls { "1" } else { "0" },
    )?;
    if let Some(password) = input.password.filter(|p| !p.trim().is_empty()) {
        set_setting(&conn, SETTING_SMTP_PASSWORD, password.trim())?;
    }
    drop(conn);
    get_smtp_settings(state)
}

#[tauri::command]
pub fn test_smtp_connection(
    state: State<'_, AppState>,
    test_recipient: String,
) -> Result<SmtpConnectionTest, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let config = load_smtp_config(&conn)?;
    let subject = "ClassMate SMTP test";
    let html = "<p>ClassMate SMTP connection test succeeded.</p>";
    match send_html_email(&config, test_recipient.trim(), subject, html) {
        Ok(()) => {
            log_email(
                &conn,
                test_recipient.trim(),
                subject,
                "test",
                "sent",
                None,
            )?;
            Ok(SmtpConnectionTest {
                ok: true,
                message: "Test email sent".into(),
            })
        }
        Err(err) => {
            log_email(
                &conn,
                test_recipient.trim(),
                subject,
                "test",
                "failed",
                Some(&err),
            )?;
            Ok(SmtpConnectionTest {
                ok: false,
                message: err,
            })
        }
    }
}

#[tauri::command]
pub fn send_parent_digest_email(
    state: State<'_, AppState>,
    input: SendParentDigestEmailInput,
) -> Result<SendParentDigestEmailResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let config = load_smtp_config(&conn)?;
    let subject_prefix = "ClassMate weekly digest";
    match send_digest_to_parent(&conn, &config, &input.parent_id, input.to_email.as_deref()) {
        Ok(recipient) => Ok(SendParentDigestEmailResult {
            recipient,
            subject: subject_prefix.into(),
            message: "Digest email sent".into(),
        }),
        Err(err) => {
            let recipient = input.to_email.unwrap_or_else(|| "unknown".into());
            log_email(
                &conn,
                &recipient,
                subject_prefix,
                "parent_digest",
                "failed",
                Some(&err),
            )?;
            Err(err)
        }
    }
}

#[tauri::command]
pub fn get_digest_email_status(state: State<'_, AppState>) -> Result<DigestEmailStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let settings = read_digest_settings(&conn);
    let last_at = parse_digest_last_at(&conn);
    let parent_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT parent_id) FROM parent_links",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(DigestEmailStatus {
        settings,
        smtp_configured: smtp_is_configured(&conn),
        parent_count,
        last_run_at: last_at.map(|dt| dt.to_rfc3339()),
        next_due_at: digest_next_due_at(&read_digest_settings(&conn), last_at),
    })
}

#[tauri::command]
pub fn set_digest_email_settings(
    state: State<'_, AppState>,
    input: DigestEmailSettings,
) -> Result<(), String> {
    if !["daily", "weekly", "off"].contains(&input.interval.as_str()) {
        return Err("Interval must be daily, weekly, or off".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        SETTING_DIGEST_ENABLED,
        if input.enabled { "true" } else { "false" },
    )?;
    set_setting(&conn, SETTING_DIGEST_INTERVAL, &input.interval)?;
    Ok(())
}

#[tauri::command]
pub fn run_scheduled_digest_now(state: State<'_, AppState>) -> Result<DigestEmailRunResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    run_all_parent_digests(&conn)
}

#[tauri::command]
pub fn list_email_log(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<EmailLogEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let max = limit.unwrap_or(50).clamp(1, 200);
    let mut stmt = conn
        .prepare(
            "SELECT id, recipient, subject, kind, status, error, created_at
             FROM email_log ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![max], |row| {
            Ok(EmailLogEntry {
                id: row.get(0)?,
                recipient: row.get(1)?,
                subject: row.get(2)?,
                kind: row.get(3)?,
                status: row.get(4)?,
                error: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
