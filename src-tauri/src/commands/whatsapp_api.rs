use crate::commands::sync::resolve_webhook_url;
use crate::commands::whatsapp::{compose_share_message, get_country_code, normalize_phone};
use crate::models::{
    CreateWhatsAppScheduledBroadcastInput, CreateNativeWhatsAppGroupInput, HubStatus,
    LinkWhatsAppGroupInput, NativeWhatsAppGroupResult, SaveWhatsAppBusinessSettingsInput,
    SaveWhatsAppComplianceSettingsInput, SaveWhatsAppInboundRoutingSettingsInput,
    SaveWhatsAppTemplateSettingsInput, SendWhatsAppBroadcastInput,
    SendWhatsAppGroupInvitesInput, SendWhatsAppTemplateBroadcastInput, WhatsAppBroadcastResult,
    WhatsAppBusinessSettings, WhatsAppComplianceSettings, WhatsAppConnectionTest,
    WhatsAppConsentLogEntry, WhatsAppConsentSnapshot, WhatsAppGdprExport, WhatsAppGroupLink,
    WhatsAppGroupRosterDiff, WhatsAppInboundMessage, WhatsAppInboundRoutingSettings,
    WhatsAppJoinRequest, WhatsAppNativeParticipant, WhatsAppOutboundMessage, WhatsAppRosterMember,
    ManageWhatsAppJoinRequestsInput, SyncNativeWhatsAppGroupRosterInput,
    SyncNativeWhatsAppGroupRosterResult, WhatsAppGroupParticipantEvent, WhatsAppGroupSettingsEvent,
    WhatsAppScheduledBroadcast, WhatsAppScheduledRunResult, WhatsAppShareInput,
    WhatsAppTemplatePreview, WhatsAppTemplateSettings,
};
use crate::AppState;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

const SETTING_API_VERSION: &str = "whatsapp_api_version";
const SETTING_PHONE_NUMBER_ID: &str = "whatsapp_phone_number_id";
const SETTING_ACCESS_TOKEN: &str = "whatsapp_access_token";
const SETTING_WEBHOOK_VERIFY_TOKEN: &str = "whatsapp_webhook_verify_token";
const SETTING_TEMPLATE_ASSIGNMENT_NAME: &str = "whatsapp_template_assignment_name";
const SETTING_TEMPLATE_ASSIGNMENT_LANGUAGE: &str = "whatsapp_template_assignment_language";
const SETTING_GROUP_INVITE_TEMPLATE_NAME: &str = "whatsapp_group_invite_template_name";
const SETTING_GROUP_INVITE_TEMPLATE_LANGUAGE: &str = "whatsapp_group_invite_template_language";
const SETTING_INBOUND_ENABLED: &str = "whatsapp_inbound_enabled";
const SETTING_INBOUND_COURSE_ID: &str = "whatsapp_inbound_course_id";
const SETTING_INBOUND_TOPIC_ID: &str = "whatsapp_inbound_topic_id";
const SETTING_AUTO_UNSUBSCRIBE: &str = "whatsapp_auto_unsubscribe";
const SETTING_UNSUBSCRIBE_KEYWORDS: &str = "whatsapp_unsubscribe_keywords";

const DEFAULT_UNSUBSCRIBE_KEYWORDS: &str = "STOP,UNSUBSCRIBE,CANCEL,OPT OUT";

const TEMPLATE_PARAM_LABELS: [&str; 4] = [
    "Student name",
    "Course title",
    "Assignment title",
    "Due date",
];

struct BusinessConfig {
    api_version: String,
    phone_number_id: String,
    access_token: String,
}

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

fn load_business_config(conn: &Connection) -> Result<BusinessConfig, String> {
    let phone_number_id = get_setting(conn, SETTING_PHONE_NUMBER_ID)
        .filter(|v| !v.trim().is_empty())
        .ok_or("WhatsApp Business API is not configured")?;
    let access_token = get_setting(conn, SETTING_ACCESS_TOKEN)
        .filter(|v| !v.trim().is_empty())
        .ok_or("WhatsApp access token is missing")?;
    let api_version = get_setting(conn, SETTING_API_VERSION).unwrap_or_else(|| "v21.0".into());
    Ok(BusinessConfig {
        api_version,
        phone_number_id,
        access_token,
    })
}

fn graph_url(config: &BusinessConfig, path: &str) -> String {
    format!(
        "https://graph.facebook.com/{}/{}/{}",
        config.api_version, config.phone_number_id, path
    )
}

fn graph_resource_url(config: &BusinessConfig, resource_id: &str, path: &str) -> String {
    format!(
        "https://graph.facebook.com/{}/{}/{}",
        config.api_version, resource_id, path
    )
}

fn graph_get(config: &BusinessConfig, url: &str) -> Result<Value, String> {
    let response = ureq::get(url)
        .set("Authorization", &format!("Bearer {}", config.access_token))
        .call()
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    if status >= 400 {
        return Err(format!("WhatsApp API error ({status}): {text}"));
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn graph_post_json(config: &BusinessConfig, url: &str, payload: Value) -> Result<Value, String> {
    let response = ureq::post(url)
        .set("Authorization", &format!("Bearer {}", config.access_token))
        .set("Content-Type", "application/json")
        .send_json(payload)
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    if status >= 400 {
        return Err(format!("WhatsApp API error ({status}): {text}"));
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn graph_delete_json(config: &BusinessConfig, url: &str, payload: Value) -> Result<Value, String> {
    let response = ureq::delete(url)
        .set("Authorization", &format!("Bearer {}", config.access_token))
        .set("Content-Type", "application/json")
        .send_json(payload)
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    if status >= 400 {
        return Err(format!("WhatsApp API error ({status}): {text}"));
    }
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn require_native_group(
    conn: &Connection,
    group_id: &str,
) -> Result<(WhatsAppGroupLink, String), String> {
    let link = read_group_link(conn, group_id)?
        .ok_or_else(|| "No WhatsApp group link record".to_string())?;
    let external_id = link
        .external_group_id
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Native group ID not set".to_string())?;
    Ok((link, external_id))
}

fn upsert_group_link(
    conn: &Connection,
    group_id: &str,
    invite_link: &str,
    external_name: Option<&str>,
    external_group_id: Option<&str>,
    native_status: &str,
    creation_error: Option<&str>,
    join_approval_mode: Option<&str>,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO whatsapp_group_links
         (id, group_id, invite_link, external_name, linked_at, external_group_id, native_status, creation_error, join_approval_mode)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(group_id) DO UPDATE SET
           invite_link = excluded.invite_link,
           external_name = COALESCE(excluded.external_name, whatsapp_group_links.external_name),
           linked_at = excluded.linked_at,
           external_group_id = COALESCE(excluded.external_group_id, whatsapp_group_links.external_group_id),
           native_status = excluded.native_status,
           creation_error = excluded.creation_error,
           join_approval_mode = COALESCE(excluded.join_approval_mode, whatsapp_group_links.join_approval_mode)",
        params![
            Uuid::new_v4().to_string(),
            group_id,
            invite_link,
            external_name,
            now,
            external_group_id,
            native_status,
            creation_error,
            join_approval_mode,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn read_group_link(conn: &Connection, group_id: &str) -> Result<Option<WhatsAppGroupLink>, String> {
    let row = conn.query_row(
        "SELECT group_id, invite_link, external_name, linked_at, external_group_id, native_status, creation_error, group_description
         FROM whatsapp_group_links WHERE group_id = ?1",
        params![group_id],
        |row| {
            Ok(WhatsAppGroupLink {
                group_id: row.get(0)?,
                invite_link: row.get(1)?,
                external_name: row.get(2)?,
                linked_at: row.get(3)?,
                external_group_id: row.get(4)?,
                native_status: row.get(5)?,
                creation_error: row.get(6)?,
                group_description: row.get(7)?,
            })
        },
    );
    match row {
        Ok(link) => Ok(Some(link)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

fn send_whatsapp_group_invite_template(
    config: &BusinessConfig,
    phone: &str,
    template_name: &str,
    language: &str,
    external_group_id: &str,
) -> Result<String, String> {
    let url = graph_url(config, "messages");
    let payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "to": phone,
        "type": "template",
        "template": {
            "name": template_name,
            "language": { "code": language },
            "components": [{
                "type": "body",
                "parameters": [{
                    "type": "group_id",
                    "group_id": external_group_id
                }]
            }]
        }
    });
    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", config.access_token))
        .set("Content-Type", "application/json")
        .send_json(payload)
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    if status >= 400 {
        return Err(format!("WhatsApp API error ({status}): {text}"));
    }
    let json: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    json["messages"][0]["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected API response: {text}"))
}

fn send_whatsapp_text(config: &BusinessConfig, phone: &str, body: &str) -> Result<String, String> {
    let url = graph_url(config, "messages");
    let payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "to": phone,
        "type": "text",
        "text": { "body": body }
    });
    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", config.access_token))
        .set("Content-Type", "application/json")
        .send_json(payload)
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    if status >= 400 {
        return Err(format!("WhatsApp API error ({status}): {text}"));
    }
    let json: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    json["messages"][0]["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected API response: {text}"))
}

fn send_whatsapp_template(
    config: &BusinessConfig,
    phone: &str,
    template_name: &str,
    language: &str,
    parameters: &[String],
) -> Result<String, String> {
    let url = graph_url(config, "messages");
    let body_params: Vec<Value> = parameters
        .iter()
        .map(|text| serde_json::json!({ "type": "text", "text": text }))
        .collect();
    let payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "to": phone,
        "type": "template",
        "template": {
            "name": template_name,
            "language": { "code": language },
            "components": [{
                "type": "body",
                "parameters": body_params
            }]
        }
    });
    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", config.access_token))
        .set("Content-Type", "application/json")
        .send_json(payload)
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    if status >= 400 {
        return Err(format!("WhatsApp API error ({status}): {text}"));
    }
    let json: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    json["messages"][0]["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected API response: {text}"))
}

fn read_template_settings(conn: &Connection) -> WhatsAppTemplateSettings {
    let assignment_name =
        get_setting(conn, SETTING_TEMPLATE_ASSIGNMENT_NAME).unwrap_or_default();
    let assignment_language = get_setting(conn, SETTING_TEMPLATE_ASSIGNMENT_LANGUAGE)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "en".into());
    let group_invite_name = get_setting(conn, SETTING_GROUP_INVITE_TEMPLATE_NAME).unwrap_or_default();
    let group_invite_language = get_setting(conn, SETTING_GROUP_INVITE_TEMPLATE_LANGUAGE)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "en".into());
    WhatsAppTemplateSettings {
        assignment_configured: !assignment_name.trim().is_empty(),
        assignment_name,
        assignment_language,
        group_invite_configured: !group_invite_name.trim().is_empty(),
        group_invite_name,
        group_invite_language,
    }
}

fn format_due_date(raw: Option<&str>) -> String {
    let Some(raw) = raw.filter(|s| !s.is_empty()) else {
        return "No due date".into();
    };
    chrono::DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.format("%b %d, %Y").to_string())
        .unwrap_or_else(|| raw.to_string())
}

fn assignment_template_context(
    conn: &Connection,
    assignment_id: &str,
) -> Result<(String, String, String), String> {
    let (course_title, title, due_at): (String, String, Option<String>) = conn
        .query_row(
            "SELECT c.title, a.title, a.due_at
             FROM assignments a
             JOIN courses c ON c.id = a.course_id
             WHERE a.id = ?1",
            params![assignment_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Assignment not found".to_string())?;
    Ok((
        course_title,
        title,
        format_due_date(due_at.as_deref()),
    ))
}

fn build_assignment_template_params(
    conn: &Connection,
    assignment_id: &str,
    student_name: &str,
) -> Result<Vec<String>, String> {
    let (course_title, title, due_at) = assignment_template_context(conn, assignment_id)?;
    Ok(vec![
        student_name.to_string(),
        course_title,
        title,
        due_at,
    ])
}

fn template_body_summary(template_name: &str, parameters: &[String]) -> String {
    format!(
        "template:{template_name} | {}",
        parameters.join(" | ")
    )
}

#[tauri::command]
pub fn get_whatsapp_template_settings(
    state: State<'_, AppState>,
) -> Result<WhatsAppTemplateSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(read_template_settings(&conn))
}

#[tauri::command]
pub fn save_whatsapp_template_settings(
    state: State<'_, AppState>,
    input: SaveWhatsAppTemplateSettingsInput,
) -> Result<WhatsAppTemplateSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        SETTING_TEMPLATE_ASSIGNMENT_NAME,
        input.assignment_name.trim(),
    )?;
    set_setting(
        &conn,
        SETTING_TEMPLATE_ASSIGNMENT_LANGUAGE,
        input
            .assignment_language
            .trim()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<String>()
            .as_str(),
    )?;
    if let Some(name) = input.group_invite_name.as_ref() {
        set_setting(
            &conn,
            SETTING_GROUP_INVITE_TEMPLATE_NAME,
            name.trim(),
        )?;
    }
    if let Some(lang) = input.group_invite_language.as_ref() {
        set_setting(
            &conn,
            SETTING_GROUP_INVITE_TEMPLATE_LANGUAGE,
            lang.trim(),
        )?;
    }
    Ok(read_template_settings(&conn))
}

#[tauri::command]
pub fn preview_whatsapp_assignment_template(
    state: State<'_, AppState>,
    assignment_id: String,
) -> Result<WhatsAppTemplatePreview, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let settings = read_template_settings(&conn);
    if !settings.assignment_configured {
        return Err("Assignment reminder template is not configured".into());
    }
    let parameters = build_assignment_template_params(&conn, &assignment_id, "Sample Student")?;
    Ok(WhatsAppTemplatePreview {
        template_name: settings.assignment_name.clone(),
        language: settings.assignment_language.clone(),
        parameters,
        parameter_labels: TEMPLATE_PARAM_LABELS.iter().map(|s| s.to_string()).collect(),
    })
}

#[tauri::command]
pub fn send_whatsapp_template_broadcast(
    state: State<'_, AppState>,
    input: SendWhatsAppTemplateBroadcastInput,
) -> Result<WhatsAppBroadcastResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    execute_template_broadcast(&conn, &input)
}

fn execute_template_broadcast(
    conn: &Connection,
    input: &SendWhatsAppTemplateBroadcastInput,
) -> Result<WhatsAppBroadcastResult, String> {
    let config = load_business_config(conn)?;
    let template_settings = read_template_settings(conn);
    if !template_settings.assignment_configured {
        return Err("Assignment reminder template is not configured".into());
    }
    let country_code = get_country_code(conn);

    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.name, u.phone, COALESCE(c.opted_in, 0)
             FROM whatsapp_group_members m
             JOIN users u ON u.id = m.user_id
             LEFT JOIN whatsapp_consent c ON c.user_id = u.id
             WHERE m.group_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let members: Vec<(String, String, Option<String>, i64)> = stmt
        .query_map(params![input.group_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let template_name = template_settings.assignment_name.clone();
    let language = template_settings.assignment_language.clone();
    let mut sent = 0i64;
    let mut failed = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (user_id, name, phone, opted_in) in members {
        if opted_in == 0 {
            skipped += 1;
            continue;
        }
        let Some(raw_phone) = phone else {
            skipped += 1;
            continue;
        };
        let Some(normalized) = normalize_phone(&raw_phone, &country_code) else {
            skipped += 1;
            errors.push(format!("Invalid phone for user {user_id}"));
            continue;
        };

        let parameters = match build_assignment_template_params(conn, &input.assignment_id, &name) {
            Ok(params) => params,
            Err(err) => {
                failed += 1;
                if errors.len() < 5 {
                    errors.push(err);
                }
                continue;
            }
        };
        let body = template_body_summary(&template_name, &parameters);
        let msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO whatsapp_outbound_messages
             (id, group_id, user_id, kind, ref_id, phone, body, status, created_at)
             VALUES (?1, ?2, ?3, 'template_assignment', ?4, ?5, ?6, 'pending', ?7)",
            params![
                msg_id,
                input.group_id,
                user_id,
                input.assignment_id,
                normalized,
                body,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

        match send_whatsapp_template(&config, &normalized, &template_name, &language, &parameters) {
            Ok(wa_message_id) => {
                let sent_at = Utc::now().to_rfc3339();
                conn.execute(
                    "UPDATE whatsapp_outbound_messages
                     SET status = 'sent', wa_message_id = ?1, sent_at = ?2
                     WHERE id = ?3",
                    params![wa_message_id, sent_at, msg_id],
                )
                .map_err(|e| e.to_string())?;
                sent += 1;
            }
            Err(err) => {
                conn.execute(
                    "UPDATE whatsapp_outbound_messages
                     SET status = 'failed', error = ?1
                     WHERE id = ?2",
                    params![err, msg_id],
                )
                .map_err(|e| e.to_string())?;
                failed += 1;
                if errors.len() < 5 {
                    errors.push(err);
                }
            }
        }
    }

    Ok(WhatsAppBroadcastResult {
        sent,
        failed,
        skipped,
        errors,
    })
}

#[tauri::command]
pub fn get_whatsapp_business_settings(
    state: State<'_, AppState>,
) -> Result<WhatsAppBusinessSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sync = state.sync.lock().map_err(|e| e.to_string())?;
    let lan_sync_url = sync.status().sync_url;
    let phone_number_id = get_setting(&conn, SETTING_PHONE_NUMBER_ID).unwrap_or_default();
    let access_token = get_setting(&conn, SETTING_ACCESS_TOKEN);
    let webhook_verify_token = get_setting(&conn, SETTING_WEBHOOK_VERIFY_TOKEN);
    let webhook_url = resolve_webhook_url(&conn, lan_sync_url);
    Ok(WhatsAppBusinessSettings {
        configured: !phone_number_id.trim().is_empty() && access_token.is_some(),
        api_version: get_setting(&conn, SETTING_API_VERSION).unwrap_or_else(|| "v21.0".into()),
        phone_number_id,
        access_token_set: access_token.filter(|t| !t.is_empty()).is_some(),
        webhook_verify_token_set: webhook_verify_token.filter(|t| !t.is_empty()).is_some(),
        webhook_url,
    })
}

#[tauri::command]
pub fn save_whatsapp_business_settings(
    state: State<'_, AppState>,
    input: SaveWhatsAppBusinessSettingsInput,
) -> Result<WhatsAppBusinessSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        SETTING_API_VERSION,
        input
            .api_version
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "v21.0".into())
            .trim(),
    )?;
    set_setting(&conn, SETTING_PHONE_NUMBER_ID, input.phone_number_id.trim())?;
    if let Some(token) = input.access_token.filter(|t| !t.trim().is_empty()) {
        set_setting(&conn, SETTING_ACCESS_TOKEN, token.trim())?;
    }
    if let Some(token) = input
        .webhook_verify_token
        .filter(|t| !t.trim().is_empty())
    {
        set_setting(&conn, SETTING_WEBHOOK_VERIFY_TOKEN, token.trim())?;
    }
    drop(conn);
    get_whatsapp_business_settings(state)
}

#[tauri::command]
pub fn test_whatsapp_business_connection(
    state: State<'_, AppState>,
) -> Result<WhatsAppConnectionTest, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let config = load_business_config(&conn)?;
    let url = format!(
        "https://graph.facebook.com/{}/{}?fields=display_phone_number,verified_name",
        config.api_version, config.phone_number_id
    );
    let response = ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", config.access_token))
        .call()
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.into_string().unwrap_or_default();
    if status >= 400 {
        return Ok(WhatsAppConnectionTest {
            ok: false,
            display_phone_number: None,
            verified_name: None,
            message: format!("Connection failed ({status}): {text}"),
        });
    }
    let json: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok(WhatsAppConnectionTest {
        ok: true,
        display_phone_number: json["display_phone_number"].as_str().map(|s| s.to_string()),
        verified_name: json["verified_name"].as_str().map(|s| s.to_string()),
        message: "Connected to WhatsApp Business API".into(),
    })
}

#[tauri::command]
pub fn link_whatsapp_group(
    state: State<'_, AppState>,
    input: LinkWhatsAppGroupInput,
) -> Result<WhatsAppGroupLink, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    upsert_group_link(
        &conn,
        &input.group_id,
        input.invite_link.trim(),
        input.external_name.as_deref(),
        None,
        "manual",
        None,
        None,
    )?;
    read_group_link(&conn, &input.group_id)?.ok_or_else(|| "Failed to read group link".into())
}

#[tauri::command]
pub fn unlink_whatsapp_group(state: State<'_, AppState>, group_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM whatsapp_group_links WHERE group_id = ?1",
        params![group_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_whatsapp_group_link(
    state: State<'_, AppState>,
    group_id: String,
) -> Result<Option<WhatsAppGroupLink>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    read_group_link(&conn, &group_id)
}

#[tauri::command]
pub fn create_native_whatsapp_group(
    state: State<'_, AppState>,
    input: CreateNativeWhatsAppGroupInput,
) -> Result<NativeWhatsAppGroupResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let config = load_business_config(&conn)?;

    let (name, member_count): (String, i64) = conn
        .query_row(
            "SELECT g.name, (SELECT COUNT(*) FROM whatsapp_group_members m WHERE m.group_id = g.id)
             FROM whatsapp_groups g WHERE g.id = ?1",
            params![input.group_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Group not found".to_string())?;

    if member_count > 8 {
        return Err("WhatsApp native groups support at most 8 participants".into());
    }

    let join_mode = input
        .join_approval_mode
        .as_deref()
        .unwrap_or("auto_approve");
    if !matches!(join_mode, "auto_approve" | "approval_required") {
        return Err("Invalid join approval mode".into());
    }

    let mut payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "subject": name,
        "join_approval_mode": join_mode,
    });
    if let Some(desc) = input.description.as_ref().filter(|d| !d.trim().is_empty()) {
        payload["description"] = Value::String(desc.trim().to_string());
    }

    let url = graph_url(&config, "groups");
    let api_result = graph_post_json(&config, &url, payload);
    let json = match api_result {
        Ok(json) => json,
        Err(err) => {
            upsert_group_link(
                &conn,
                &input.group_id,
                "",
                Some(&name),
                None,
                "failed",
                Some(&err),
                Some(join_mode),
            )?;
            return Ok(NativeWhatsAppGroupResult {
                group_id: input.group_id,
                external_group_id: None,
                invite_link: None,
                native_status: "failed".into(),
                message: err,
            });
        }
    };

    let external_group_id = json["id"].as_str().map(|s| s.to_string());
    let invite_link = json["invite_link"].as_str().map(|s| s.to_string());
    let native_status = if invite_link.is_some() {
        "active"
    } else {
        "pending"
    };

    upsert_group_link(
        &conn,
        &input.group_id,
        invite_link.as_deref().unwrap_or(""),
        Some(&name),
        external_group_id.as_deref(),
        native_status,
        None,
        Some(join_mode),
    )?;

    Ok(NativeWhatsAppGroupResult {
        group_id: input.group_id,
        external_group_id,
        invite_link,
        native_status: native_status.into(),
        message: if native_status == "pending" {
            "Group created. Invite link will arrive via webhook, or use Refresh link.".into()
        } else {
            "Native WhatsApp group created.".into()
        },
    })
}

#[tauri::command]
pub fn refresh_native_whatsapp_group(
    state: State<'_, AppState>,
    group_id: String,
) -> Result<NativeWhatsAppGroupResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let config = load_business_config(&conn)?;
    let link = read_group_link(&conn, &group_id)?
        .ok_or_else(|| "No WhatsApp group link record".to_string())?;
    let external_id = link
        .external_group_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Native group ID not set. Create a native group first.".to_string())?;

    let url = graph_resource_url(&config, external_id, "invite_link");
    let json = match graph_get(&config, &url) {
        Ok(json) => json,
        Err(err) => {
            upsert_group_link(
                &conn,
                &group_id,
                link.invite_link.as_str(),
                link.external_name.as_deref(),
                Some(external_id),
                "pending",
                Some(&err),
                None,
            )?;
            return Err(err);
        }
    };

    let invite_link = json["invite_link"]
        .as_str()
        .or_else(|| json["link"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No invite link in API response".to_string())?;

    upsert_group_link(
        &conn,
        &group_id,
        &invite_link,
        link.external_name.as_deref(),
        Some(external_id),
        "active",
        None,
        None,
    )?;

    Ok(NativeWhatsAppGroupResult {
        group_id,
        external_group_id: Some(external_id.to_string()),
        invite_link: Some(invite_link),
        native_status: "active".into(),
        message: "Invite link refreshed.".into(),
    })
}

#[tauri::command]
pub fn send_whatsapp_group_invites(
    state: State<'_, AppState>,
    input: SendWhatsAppGroupInvitesInput,
) -> Result<WhatsAppBroadcastResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let config = load_business_config(&conn)?;
    let template_settings = read_template_settings(&conn);
    if !template_settings.group_invite_configured {
        return Err("Group invite template is not configured".into());
    }

    let link = read_group_link(&conn, &input.group_id)?
        .ok_or_else(|| "No WhatsApp group link record".to_string())?;
    let external_group_id = link
        .external_group_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Native group ID not set".to_string())?;

    let country_code = get_country_code(&conn);
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.name, u.phone, COALESCE(c.opted_in, 0)
             FROM whatsapp_group_members m
             JOIN users u ON u.id = m.user_id
             LEFT JOIN whatsapp_consent c ON c.user_id = u.id
             WHERE m.group_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let members: Vec<(String, String, Option<String>, i64)> = stmt
        .query_map(params![input.group_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let template_name = template_settings.group_invite_name.clone();
    let language = template_settings.group_invite_language.clone();
    let mut sent = 0i64;
    let mut failed = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (user_id, _name, phone, opted_in) in members {
        if opted_in == 0 {
            skipped += 1;
            continue;
        }
        let Some(raw_phone) = phone else {
            skipped += 1;
            continue;
        };
        let Some(normalized) = normalize_phone(&raw_phone, &country_code) else {
            skipped += 1;
            errors.push(format!("Invalid phone for user {user_id}"));
            continue;
        };

        let body = format!("Group invite template: {template_name}");
        let msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO whatsapp_outbound_messages
             (id, group_id, user_id, kind, ref_id, phone, body, status, created_at)
             VALUES (?1, ?2, ?3, 'template_group_invite', ?4, ?5, ?6, 'pending', ?7)",
            params![
                msg_id,
                input.group_id,
                user_id,
                external_group_id,
                normalized,
                body,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

        match send_whatsapp_group_invite_template(
            &config,
            &normalized,
            &template_name,
            &language,
            external_group_id,
        ) {
            Ok(wa_id) => {
                conn.execute(
                    "UPDATE whatsapp_outbound_messages
                     SET status = 'sent', wa_message_id = ?1, sent_at = ?2
                     WHERE id = ?3",
                    params![wa_id, now, msg_id],
                )
                .map_err(|e| e.to_string())?;
                sent += 1;
            }
            Err(err) => {
                conn.execute(
                    "UPDATE whatsapp_outbound_messages
                     SET status = 'failed', error = ?1
                     WHERE id = ?2",
                    params![err, msg_id],
                )
                .map_err(|e| e.to_string())?;
                failed += 1;
                if errors.len() < 5 {
                    errors.push(err);
                }
            }
        }
    }

    Ok(WhatsAppBroadcastResult {
        sent,
        failed,
        skipped,
        errors,
    })
}

fn phone_tail(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .take(10)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

fn phones_match(a: &str, b: &str) -> bool {
    let ta = phone_tail(a);
    let tb = phone_tail(b);
    !ta.is_empty() && ta == tb
}

fn parse_participants_from_json(json: &Value) -> Vec<WhatsAppNativeParticipant> {
    let mut participants = Vec::new();
    if let Some(items) = json["participants"].as_array() {
        for item in items {
            if let Some(wa_id) = item["wa_id"].as_str() {
                participants.push(WhatsAppNativeParticipant {
                    wa_id: wa_id.to_string(),
                });
            }
        }
    } else if let Some(data) = json["participants"]["data"].as_array() {
        for item in data {
            if let Some(wa_id) = item["wa_id"].as_str() {
                participants.push(WhatsAppNativeParticipant {
                    wa_id: wa_id.to_string(),
                });
            }
        }
    }
    participants
}

fn fetch_group_participants_from_api(
    config: &BusinessConfig,
    external_group_id: &str,
) -> Result<Vec<WhatsAppNativeParticipant>, String> {
    let url = format!(
        "https://graph.facebook.com/{}/{}?fields=participants",
        config.api_version, external_group_id
    );
    let json = graph_get(config, &url)?;
    Ok(parse_participants_from_json(&json))
}

fn lookup_classmate_group_id(conn: &Connection, external_group_id: &str) -> Option<String> {
    conn.query_row(
        "SELECT group_id FROM whatsapp_group_links WHERE external_group_id = ?1",
        params![external_group_id],
        |row| row.get(0),
    )
    .ok()
}

fn cache_group_participants(
    conn: &Connection,
    external_group_id: &str,
    participants: &[WhatsAppNativeParticipant],
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    let json = serde_json::to_string(participants).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE whatsapp_group_links
         SET cached_participants_json = ?1, roster_synced_at = ?2
         WHERE external_group_id = ?3",
        params![json, now, external_group_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn refresh_cached_group_participants(
    conn: &Connection,
    external_group_id: &str,
) -> Result<(), String> {
    let config = load_business_config(conn)?;
    let participants = fetch_group_participants_from_api(&config, external_group_id)?;
    cache_group_participants(conn, external_group_id, &participants)
}

fn read_cached_participants(
    conn: &Connection,
    group_id: &str,
) -> Result<Option<Vec<WhatsAppNativeParticipant>>, String> {
    let json_text: Option<String> = conn
        .query_row(
            "SELECT cached_participants_json FROM whatsapp_group_links WHERE group_id = ?1",
            params![group_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    let Some(json_text) = json_text.filter(|s| !s.trim().is_empty()) else {
        return Ok(None);
    };
    let participants: Vec<WhatsAppNativeParticipant> =
        serde_json::from_str(&json_text).map_err(|e| e.to_string())?;
    Ok(Some(participants))
}

fn insert_participant_event(
    conn: &Connection,
    classmate_group_id: Option<&str>,
    external_group_id: &str,
    event_type: &str,
    direction: Option<&str>,
    wa_id: Option<&str>,
    reason: Option<&str>,
    join_request_id: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO whatsapp_group_participant_events
         (id, group_id, external_group_id, event_type, direction, wa_id, reason, join_request_id, received_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            Uuid::new_v4().to_string(),
            classmate_group_id,
            external_group_id,
            event_type,
            direction,
            wa_id,
            reason,
            join_request_id,
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn participant_wa_id(item: &WebhookParticipantRef) -> Option<String> {
    item.wa_id
        .as_deref()
        .or(item.input.as_deref())
        .map(|s| s.to_string())
}

#[tauri::command]
pub fn list_whatsapp_group_participant_events(
    state: State<'_, AppState>,
    group_id: String,
    limit: Option<i64>,
) -> Result<Vec<WhatsAppGroupParticipantEvent>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(30).clamp(1, 200);
    let mut stmt = conn
        .prepare(
            "SELECT id, group_id, external_group_id, event_type, direction, wa_id, reason, join_request_id, received_at
             FROM whatsapp_group_participant_events
             WHERE group_id = ?1
             ORDER BY received_at DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![group_id, limit], |row| {
            Ok(WhatsAppGroupParticipantEvent {
                id: row.get(0)?,
                group_id: row.get(1)?,
                external_group_id: row.get(2)?,
                event_type: row.get(3)?,
                direction: row.get(4)?,
                wa_id: row.get(5)?,
                reason: row.get(6)?,
                join_request_id: row.get(7)?,
                received_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn insert_settings_event(
    conn: &Connection,
    classmate_group_id: Option<&str>,
    external_group_id: &str,
    event_type: &str,
    setting_kind: Option<&str>,
    setting_value: Option<&str>,
    update_successful: Option<bool>,
    error_summary: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO whatsapp_group_settings_events
         (id, group_id, external_group_id, event_type, setting_kind, setting_value, update_successful, error_summary, received_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            Uuid::new_v4().to_string(),
            classmate_group_id,
            external_group_id,
            event_type,
            setting_kind,
            setting_value,
            update_successful.map(|v| if v { 1i64 } else { 0i64 }),
            error_summary,
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_whatsapp_group_settings_events(
    state: State<'_, AppState>,
    group_id: String,
    limit: Option<i64>,
) -> Result<Vec<WhatsAppGroupSettingsEvent>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(30).clamp(1, 200);
    let mut stmt = conn
        .prepare(
            "SELECT id, group_id, external_group_id, event_type, setting_kind, setting_value, update_successful, error_summary, received_at
             FROM whatsapp_group_settings_events
             WHERE group_id = ?1
             ORDER BY received_at DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![group_id, limit], |row| {
            let successful: Option<i64> = row.get(6)?;
            Ok(WhatsAppGroupSettingsEvent {
                id: row.get(0)?,
                group_id: row.get(1)?,
                external_group_id: row.get(2)?,
                event_type: row.get(3)?,
                setting_kind: row.get(4)?,
                setting_value: row.get(5)?,
                update_successful: successful.map(|v| v != 0),
                error_summary: row.get(7)?,
                received_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn get_whatsapp_group_roster_diff(
    state: State<'_, AppState>,
    group_id: String,
) -> Result<WhatsAppGroupRosterDiff, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    compute_whatsapp_group_roster_diff(&conn, &group_id)
}

fn compute_whatsapp_group_roster_diff(
    conn: &Connection,
    group_id: &str,
) -> Result<WhatsAppGroupRosterDiff, String> {
    let country_code = get_country_code(conn);

    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.name, u.phone
             FROM whatsapp_group_members m
             JOIN users u ON u.id = m.user_id
             WHERE m.group_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let classmate_members: Vec<WhatsAppRosterMember> = stmt
        .query_map(params![group_id], |row| {
            let phone: Option<String> = row.get(2)?;
            let normalized = phone
                .as_ref()
                .and_then(|p| normalize_phone(p, &country_code));
            Ok(WhatsAppRosterMember {
                user_id: row.get(0)?,
                name: row.get(1)?,
                phone,
                normalized_phone: normalized,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let link = read_group_link(conn, group_id)?;
    let Some(external_id) = link
        .and_then(|l| l.external_group_id)
        .filter(|s| !s.is_empty())
    else {
        return Ok(WhatsAppGroupRosterDiff {
            group_id: group_id.to_string(),
            native_available: false,
            classmate_members: classmate_members.clone(),
            whatsapp_participants: vec![],
            matched_count: 0,
            only_in_classmate: classmate_members,
            only_in_whatsapp: vec![],
            message: "No native WhatsApp group linked. Link or create a native group first.".into(),
        });
    };

    let config = match load_business_config(conn) {
        Ok(cfg) => cfg,
        Err(err) => {
            return Ok(WhatsAppGroupRosterDiff {
                group_id: group_id.to_string(),
                native_available: false,
                classmate_members: classmate_members.clone(),
                whatsapp_participants: vec![],
                matched_count: 0,
                only_in_classmate: classmate_members,
                only_in_whatsapp: vec![],
                message: err,
            });
        }
    };

    let whatsapp_participants = if let Ok(Some(cached)) = read_cached_participants(conn, group_id) {
        cached
    } else {
        match fetch_group_participants_from_api(&config, &external_id) {
            Ok(participants) => {
                let _ = cache_group_participants(conn, &external_id, &participants);
                participants
            }
            Err(err) => {
                return Ok(WhatsAppGroupRosterDiff {
                    group_id: group_id.to_string(),
                    native_available: false,
                    classmate_members: classmate_members.clone(),
                    whatsapp_participants: vec![],
                    matched_count: 0,
                    only_in_classmate: classmate_members,
                    only_in_whatsapp: vec![],
                    message: err,
                });
            }
        }
    };

    let mut matched = 0i64;
    let mut only_in_classmate = Vec::new();
    for member in &classmate_members {
        let phone = member
            .normalized_phone
            .as_deref()
            .or(member.phone.as_deref());
        let is_matched = phone.is_some_and(|p| {
            whatsapp_participants
                .iter()
                .any(|part| phones_match(p, &part.wa_id))
        });
        if is_matched {
            matched += 1;
        } else {
            only_in_classmate.push(member.clone());
        }
    }

    let mut only_in_whatsapp = Vec::new();
    for participant in &whatsapp_participants {
        let is_matched = classmate_members.iter().any(|member| {
            member
                .normalized_phone
                .as_deref()
                .or(member.phone.as_deref())
                .is_some_and(|p| phones_match(p, &participant.wa_id))
        });
        if !is_matched {
            only_in_whatsapp.push(participant.clone());
        }
    }

    let message = if only_in_classmate.is_empty() && only_in_whatsapp.is_empty() {
        "Roster matches native WhatsApp group.".into()
    } else {
        format!(
            "{} matched, {} only in ClassMate, {} only in WhatsApp.",
            matched,
            only_in_classmate.len(),
            only_in_whatsapp.len()
        )
    };

    let cache_note = if read_cached_participants(conn, group_id)?.is_some() {
        " (cached roster)"
    } else {
        ""
    };

    Ok(WhatsAppGroupRosterDiff {
        group_id: group_id.to_string(),
        native_available: true,
        classmate_members,
        whatsapp_participants,
        matched_count: matched,
        only_in_classmate,
        only_in_whatsapp,
        message: format!("{message}{cache_note}"),
    })
}

fn send_group_invites_for_users(
    conn: &Connection,
    config: &BusinessConfig,
    group_id: &str,
    external_group_id: &str,
    user_ids: &[String],
) -> Result<WhatsAppBroadcastResult, String> {
    let template_settings = read_template_settings(conn);
    if !template_settings.group_invite_configured {
        return Err("Group invite template is not configured".into());
    }
    if user_ids.is_empty() {
        return Ok(WhatsAppBroadcastResult {
            sent: 0,
            failed: 0,
            skipped: 0,
            errors: vec![],
        });
    }

    let targets: std::collections::HashSet<String> = user_ids.iter().cloned().collect();
    let country_code = get_country_code(conn);
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.name, u.phone, COALESCE(c.opted_in, 0)
             FROM whatsapp_group_members m
             JOIN users u ON u.id = m.user_id
             LEFT JOIN whatsapp_consent c ON c.user_id = u.id
             WHERE m.group_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let members: Vec<(String, String, Option<String>, i64)> = stmt
        .query_map(params![group_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let template_name = template_settings.group_invite_name.clone();
    let language = template_settings.group_invite_language.clone();
    let mut sent = 0i64;
    let mut failed = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (user_id, _name, phone, opted_in) in members {
        if !targets.contains(&user_id) {
            continue;
        }
        if opted_in == 0 {
            skipped += 1;
            continue;
        }
        let Some(raw_phone) = phone else {
            skipped += 1;
            continue;
        };
        let Some(normalized) = normalize_phone(&raw_phone, &country_code) else {
            skipped += 1;
            errors.push(format!("Invalid phone for user {user_id}"));
            continue;
        };

        let body = format!("Group invite template: {template_name}");
        let msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO whatsapp_outbound_messages
             (id, group_id, user_id, kind, ref_id, phone, body, status, created_at)
             VALUES (?1, ?2, ?3, 'template_group_invite', ?4, ?5, ?6, 'pending', ?7)",
            params![
                msg_id,
                group_id,
                user_id,
                external_group_id,
                normalized,
                body,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

        match send_whatsapp_group_invite_template(
            config,
            &normalized,
            &template_name,
            &language,
            external_group_id,
        ) {
            Ok(wa_id) => {
                conn.execute(
                    "UPDATE whatsapp_outbound_messages
                     SET status = 'sent', wa_message_id = ?1, sent_at = ?2
                     WHERE id = ?3",
                    params![wa_id, now, msg_id],
                )
                .map_err(|e| e.to_string())?;
                sent += 1;
            }
            Err(err) => {
                conn.execute(
                    "UPDATE whatsapp_outbound_messages
                     SET status = 'failed', error = ?1
                     WHERE id = ?2",
                    params![err, msg_id],
                )
                .map_err(|e| e.to_string())?;
                failed += 1;
                if errors.len() < 5 {
                    errors.push(err);
                }
            }
        }
    }

    Ok(WhatsAppBroadcastResult {
        sent,
        failed,
        skipped,
        errors,
    })
}

fn remove_whatsapp_participants(
    config: &BusinessConfig,
    external_group_id: &str,
    wa_ids: &[String],
) -> Result<(), String> {
    if wa_ids.is_empty() {
        return Ok(());
    }
    let participants: Vec<Value> = wa_ids
        .iter()
        .map(|id| serde_json::json!({ "user": id }))
        .collect();
    let url = graph_resource_url(config, external_group_id, "participants");
    let payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "participants": participants
    });
    graph_delete_json(config, &url, payload).map(|_| ())
}

fn parse_join_requests(json: &Value) -> Vec<WhatsAppJoinRequest> {
    let mut requests = Vec::new();
    let items = json["data"]
        .as_array()
        .or_else(|| json["join_requests"].as_array());
    if let Some(items) = items {
        for item in items {
            if let Some(id) = item["join_request_id"].as_str() {
                requests.push(WhatsAppJoinRequest {
                    join_request_id: id.to_string(),
                    wa_id: item["wa_id"].as_str().unwrap_or("").to_string(),
                    creation_timestamp: item["creation_timestamp"].as_str().map(|s| s.to_string()),
                });
            }
        }
    }
    requests
}

#[tauri::command]
pub fn list_whatsapp_group_join_requests(
    state: State<'_, AppState>,
    group_id: String,
) -> Result<Vec<WhatsAppJoinRequest>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (_, external_id) = require_native_group(&conn, &group_id)?;
    let config = load_business_config(&conn)?;
    let url = graph_resource_url(&config, &external_id, "join_requests");
    let json = graph_get(&config, &url)?;
    Ok(parse_join_requests(&json))
}

#[tauri::command]
pub fn approve_whatsapp_group_join_requests(
    state: State<'_, AppState>,
    input: ManageWhatsAppJoinRequestsInput,
) -> Result<WhatsAppBroadcastResult, String> {
    if input.join_request_ids.is_empty() {
        return Err("No join request IDs provided".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (_, external_id) = require_native_group(&conn, &input.group_id)?;
    let config = load_business_config(&conn)?;
    let url = graph_resource_url(&config, &external_id, "join_requests");
    let payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "join_requests": input.join_request_ids
    });
    graph_post_json(&config, &url, payload)?;
    Ok(WhatsAppBroadcastResult {
        sent: input.join_request_ids.len() as i64,
        failed: 0,
        skipped: 0,
        errors: vec![],
    })
}

#[tauri::command]
pub fn reject_whatsapp_group_join_requests(
    state: State<'_, AppState>,
    input: ManageWhatsAppJoinRequestsInput,
) -> Result<WhatsAppBroadcastResult, String> {
    if input.join_request_ids.is_empty() {
        return Err("No join request IDs provided".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (_, external_id) = require_native_group(&conn, &input.group_id)?;
    let config = load_business_config(&conn)?;
    let url = graph_resource_url(&config, &external_id, "join_requests");
    let payload = serde_json::json!({
        "messaging_product": "whatsapp",
        "join_requests": input.join_request_ids
    });
    graph_delete_json(&config, &url, payload)?;
    Ok(WhatsAppBroadcastResult {
        sent: input.join_request_ids.len() as i64,
        failed: 0,
        skipped: 0,
        errors: vec![],
    })
}

#[tauri::command]
pub fn sync_native_whatsapp_group_roster(
    state: State<'_, AppState>,
    input: SyncNativeWhatsAppGroupRosterInput,
) -> Result<SyncNativeWhatsAppGroupRosterResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let diff = compute_whatsapp_group_roster_diff(&conn, &input.group_id)?;
    if !diff.native_available {
        return Ok(SyncNativeWhatsAppGroupRosterResult {
            diff: diff.clone(),
            invites: WhatsAppBroadcastResult {
                sent: 0,
                failed: 0,
                skipped: 0,
                errors: vec![],
            },
            removed: 0,
            remove_errors: vec![],
            message: diff.message.clone(),
        });
    }

    let (_, external_id) = require_native_group(&conn, &input.group_id)?;
    let config = load_business_config(&conn)?;

    let invites = if input.send_invites {
        let missing_ids: Vec<String> = diff
            .only_in_classmate
            .iter()
            .map(|m| m.user_id.clone())
            .collect();
        send_group_invites_for_users(
            &conn,
            &config,
            &input.group_id,
            &external_id,
            &missing_ids,
        )?
    } else {
        WhatsAppBroadcastResult {
            sent: 0,
            failed: 0,
            skipped: 0,
            errors: vec![],
        }
    };

    let mut removed = 0i64;
    let mut remove_errors = Vec::new();
    if input.remove_orphans && !diff.only_in_whatsapp.is_empty() {
        let wa_ids: Vec<String> = diff
            .only_in_whatsapp
            .iter()
            .map(|p| p.wa_id.clone())
            .collect();
        match remove_whatsapp_participants(&config, &external_id, &wa_ids) {
            Ok(()) => removed = wa_ids.len() as i64,
            Err(err) => remove_errors.push(err),
        }
    }

    let _ = refresh_cached_group_participants(&conn, &external_id);

    let refreshed = compute_whatsapp_group_roster_diff(&conn, &input.group_id)?;
    let message = format!(
        "Invites sent: {}. Removed from WhatsApp: {}. {}",
        invites.sent, removed, refreshed.message
    );

    Ok(SyncNativeWhatsAppGroupRosterResult {
        diff: refreshed,
        invites,
        removed,
        remove_errors,
        message,
    })
}

#[tauri::command]
pub fn set_whatsapp_consent(
    state: State<'_, AppState>,
    user_id: String,
    opted_in: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    apply_whatsapp_consent(&conn, &user_id, opted_in, "admin", None)
}

fn write_consent_log(
    conn: &Connection,
    user_id: &str,
    opted_in: bool,
    source: &str,
    note: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO whatsapp_consent_log (id, user_id, opted_in, source, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            Uuid::new_v4().to_string(),
            user_id,
            opted_in as i64,
            source,
            note,
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn apply_whatsapp_consent(
    conn: &Connection,
    user_id: &str,
    opted_in: bool,
    source: &str,
    note: Option<&str>,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO whatsapp_consent (user_id, opted_in, opted_in_at, source)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(user_id) DO UPDATE SET
           opted_in = excluded.opted_in,
           opted_in_at = excluded.opted_in_at,
           source = excluded.source",
        params![user_id, opted_in as i64, now, source],
    )
    .map_err(|e| e.to_string())?;
    write_consent_log(conn, user_id, opted_in, source, note)
}

fn read_compliance_settings(conn: &Connection) -> WhatsAppComplianceSettings {
    let auto_unsubscribe = get_setting(conn, SETTING_AUTO_UNSUBSCRIBE)
        .map(|v| v == "true")
        .unwrap_or(true);
    let unsubscribe_keywords = get_setting(conn, SETTING_UNSUBSCRIBE_KEYWORDS)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_UNSUBSCRIBE_KEYWORDS.into());
    WhatsAppComplianceSettings {
        auto_unsubscribe,
        unsubscribe_keywords,
    }
}

fn is_unsubscribe_message(conn: &Connection, body: &str) -> bool {
    let settings = read_compliance_settings(conn);
    if !settings.auto_unsubscribe {
        return false;
    }
    let normalized = body.trim().to_uppercase();
    settings
        .unsubscribe_keywords
        .split(',')
        .map(|k| k.trim().to_uppercase())
        .filter(|k| !k.is_empty())
        .any(|keyword| normalized == keyword || normalized.starts_with(&format!("{keyword} ")))
}

fn maybe_process_unsubscribe(conn: &Connection, user_id: &str, body: &str) -> Result<bool, String> {
    if !is_unsubscribe_message(conn, body) {
        return Ok(false);
    }
    apply_whatsapp_consent(
        conn,
        user_id,
        false,
        "inbound_unsubscribe",
        Some(body.trim()),
    )?;
    Ok(true)
}

#[tauri::command]
pub fn get_whatsapp_compliance_settings(
    state: State<'_, AppState>,
) -> Result<WhatsAppComplianceSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(read_compliance_settings(&conn))
}

#[tauri::command]
pub fn save_whatsapp_compliance_settings(
    state: State<'_, AppState>,
    input: SaveWhatsAppComplianceSettingsInput,
) -> Result<WhatsAppComplianceSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        SETTING_AUTO_UNSUBSCRIBE,
        if input.auto_unsubscribe { "true" } else { "false" },
    )?;
    set_setting(
        &conn,
        SETTING_UNSUBSCRIBE_KEYWORDS,
        input.unsubscribe_keywords.trim(),
    )?;
    Ok(read_compliance_settings(&conn))
}

#[tauri::command]
pub fn export_whatsapp_gdpr(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<WhatsAppGdprExport, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (user_name, email, phone): (String, String, Option<String>) = conn
        .query_row(
            "SELECT name, email, phone FROM users WHERE id = ?1",
            params![user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "User not found".to_string())?;

    let consent = conn
        .query_row(
            "SELECT opted_in, opted_in_at, source FROM whatsapp_consent WHERE user_id = ?1",
            params![user_id],
            |row| {
                Ok(WhatsAppConsentSnapshot {
                    opted_in: row.get::<_, i64>(0)? != 0,
                    opted_in_at: row.get(1)?,
                    source: row.get(2)?,
                })
            },
        )
        .ok();

    let mut log_stmt = conn
        .prepare(
            "SELECT id, user_id, opted_in, source, note, created_at
             FROM whatsapp_consent_log WHERE user_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let consent_log = log_stmt
        .query_map(params![user_id], |row| {
            Ok(WhatsAppConsentLogEntry {
                id: row.get(0)?,
                user_id: row.get(1)?,
                opted_in: row.get::<_, i64>(2)? != 0,
                source: row.get(3)?,
                note: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut groups_stmt = conn
        .prepare(
            "SELECT g.name || ' (' || c.title || ')' AS label
             FROM whatsapp_group_members m
             JOIN whatsapp_groups g ON g.id = m.group_id
             JOIN courses c ON c.id = g.course_id
             WHERE m.user_id = ?1
             ORDER BY c.title, g.name",
        )
        .map_err(|e| e.to_string())?;
    let groups = groups_stmt
        .query_map(params![user_id], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut outbound_stmt = conn
        .prepare(
            "SELECT m.id, m.group_id, m.user_id, u.name, m.kind, m.ref_id, m.phone, m.body,
                    m.status, m.wa_message_id, m.error, m.created_at, m.sent_at, m.delivered_at, m.read_at
             FROM whatsapp_outbound_messages m
             LEFT JOIN users u ON u.id = m.user_id
             WHERE m.user_id = ?1
             ORDER BY m.created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let outbound_messages = outbound_stmt
        .query_map(params![user_id], |row| {
            Ok(WhatsAppOutboundMessage {
                id: row.get(0)?,
                group_id: row.get(1)?,
                user_id: row.get(2)?,
                user_name: row.get(3)?,
                kind: row.get(4)?,
                ref_id: row.get(5)?,
                phone: row.get(6)?,
                body: row.get(7)?,
                status: row.get(8)?,
                wa_message_id: row.get(9)?,
                error: row.get(10)?,
                created_at: row.get(11)?,
                sent_at: row.get(12)?,
                delivered_at: row.get(13)?,
                read_at: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut inbound_stmt = conn
        .prepare(
            "SELECT id, wa_message_id, from_phone, from_user_id, from_user_name, body, status,
                    routed_topic_id, routed_post_id, received_at
             FROM whatsapp_inbound_messages
             WHERE from_user_id = ?1
             ORDER BY received_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let inbound_messages = inbound_stmt
        .query_map(params![user_id], |row| {
            Ok(WhatsAppInboundMessage {
                id: row.get(0)?,
                wa_message_id: row.get(1)?,
                from_phone: row.get(2)?,
                from_user_id: row.get(3)?,
                from_user_name: row.get(4)?,
                body: row.get(5)?,
                status: row.get(6)?,
                routed_topic_id: row.get(7)?,
                routed_post_id: row.get(8)?,
                received_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(WhatsAppGdprExport {
        exported_at: Utc::now().to_rfc3339(),
        user_id,
        user_name,
        email,
        phone,
        consent,
        consent_log,
        groups,
        outbound_messages,
        inbound_messages,
    })
}

#[tauri::command]
pub fn list_whatsapp_consent_log(
    state: State<'_, AppState>,
    user_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<WhatsAppConsentLogEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let max = limit.unwrap_or(100).clamp(1, 500);
    let sql = if user_id.as_ref().is_some_and(|s| !s.is_empty()) {
        "SELECT id, user_id, opted_in, source, note, created_at
         FROM whatsapp_consent_log WHERE user_id = ?1
         ORDER BY created_at DESC LIMIT ?2"
    } else {
        "SELECT id, user_id, opted_in, source, note, created_at
         FROM whatsapp_consent_log ORDER BY created_at DESC LIMIT ?1"
    };

    let rows = if let Some(ref uid) = user_id.filter(|s| !s.is_empty()) {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map(params![uid, max], |row| {
                Ok(WhatsAppConsentLogEntry {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    opted_in: row.get::<_, i64>(2)? != 0,
                    source: row.get(3)?,
                    note: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map(params![max], |row| {
                Ok(WhatsAppConsentLogEntry {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    opted_in: row.get::<_, i64>(2)? != 0,
                    source: row.get(3)?,
                    note: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(rows)
}

#[tauri::command]
pub fn send_whatsapp_broadcast(
    state: State<'_, AppState>,
    input: SendWhatsAppBroadcastInput,
) -> Result<WhatsAppBroadcastResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let hub_status = state.hub.lock().map_err(|e| e.to_string())?.status();
    execute_text_broadcast(&conn, Some(hub_status), &input)
}

fn execute_text_broadcast(
    conn: &Connection,
    hub_status: Option<HubStatus>,
    input: &SendWhatsAppBroadcastInput,
) -> Result<WhatsAppBroadcastResult, String> {
    let config = load_business_config(conn)?;
    let country_code = get_country_code(conn);
    let share_input = WhatsAppShareInput {
        kind: input.kind.clone(),
        assignment_id: input.assignment_id.clone(),
        announcement_id: input.announcement_id.clone(),
        group_id: Some(input.group_id.clone()),
        custom_message: input.custom_message.clone(),
    };
    let message = compose_share_message(conn, hub_status, &share_input)?;

    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.name, u.phone, COALESCE(c.opted_in, 0)
             FROM whatsapp_group_members m
             JOIN users u ON u.id = m.user_id
             LEFT JOIN whatsapp_consent c ON c.user_id = u.id
             WHERE m.group_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let members: Vec<(String, String, Option<String>, i64)> = stmt
        .query_map(params![input.group_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let ref_id = input
        .assignment_id
        .clone()
        .or_else(|| input.announcement_id.clone())
        .or_else(|| input.custom_message.clone().map(|_| input.group_id.clone()));

    let mut sent = 0i64;
    let mut failed = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (user_id, _name, phone, opted_in) in members {
        if opted_in == 0 {
            skipped += 1;
            continue;
        }
        let Some(raw_phone) = phone else {
            skipped += 1;
            continue;
        };
        let Some(normalized) = normalize_phone(&raw_phone, &country_code) else {
            skipped += 1;
            errors.push(format!("Invalid phone for user {user_id}"));
            continue;
        };

        let msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO whatsapp_outbound_messages
             (id, group_id, user_id, kind, ref_id, phone, body, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8)",
            params![
                msg_id,
                input.group_id,
                user_id,
                input.kind,
                ref_id,
                normalized,
                message,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

        match send_whatsapp_text(&config, &normalized, &message) {
            Ok(wa_message_id) => {
                let sent_at = Utc::now().to_rfc3339();
                conn.execute(
                    "UPDATE whatsapp_outbound_messages
                     SET status = 'sent', wa_message_id = ?1, sent_at = ?2
                     WHERE id = ?3",
                    params![wa_message_id, sent_at, msg_id],
                )
                .map_err(|e| e.to_string())?;
                sent += 1;
            }
            Err(err) => {
                conn.execute(
                    "UPDATE whatsapp_outbound_messages
                     SET status = 'failed', error = ?1
                     WHERE id = ?2",
                    params![err, msg_id],
                )
                .map_err(|e| e.to_string())?;
                failed += 1;
                errors.push(err);
            }
        }
    }

    Ok(WhatsAppBroadcastResult {
        sent,
        failed,
        skipped,
        errors,
    })
}

#[tauri::command]
pub fn list_whatsapp_outbound_messages(
    state: State<'_, AppState>,
    group_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<WhatsAppOutboundMessage>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let max = limit.unwrap_or(50).clamp(1, 200);
    let sql = if group_id.is_some() {
        "SELECT m.id, m.group_id, m.user_id, u.name, m.kind, m.ref_id, m.phone, m.body,
                m.status, m.wa_message_id, m.error, m.created_at, m.sent_at, m.delivered_at, m.read_at
         FROM whatsapp_outbound_messages m
         LEFT JOIN users u ON u.id = m.user_id
         WHERE m.group_id = ?1
         ORDER BY m.created_at DESC
         LIMIT ?2"
    } else {
        "SELECT m.id, m.group_id, m.user_id, u.name, m.kind, m.ref_id, m.phone, m.body,
                m.status, m.wa_message_id, m.error, m.created_at, m.sent_at, m.delivered_at, m.read_at
         FROM whatsapp_outbound_messages m
         LEFT JOIN users u ON u.id = m.user_id
         ORDER BY m.created_at DESC
         LIMIT ?1"
    };

    let map_row = |row: &rusqlite::Row<'_>| {
        Ok(WhatsAppOutboundMessage {
            id: row.get(0)?,
            group_id: row.get(1)?,
            user_id: row.get(2)?,
            user_name: row.get(3)?,
            kind: row.get(4)?,
            ref_id: row.get(5)?,
            phone: row.get(6)?,
            body: row.get(7)?,
            status: row.get(8)?,
            wa_message_id: row.get(9)?,
            error: row.get(10)?,
            created_at: row.get(11)?,
            sent_at: row.get(12)?,
            delivered_at: row.get(13)?,
            read_at: row.get(14)?,
        })
    };

    let messages = if let Some(ref gid) = group_id {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![gid, max], map_row)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![max], map_row)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(messages)
}

fn read_inbound_routing_settings(conn: &Connection) -> WhatsAppInboundRoutingSettings {
    let enabled = get_setting(conn, SETTING_INBOUND_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false);
    let course_id = get_setting(conn, SETTING_INBOUND_COURSE_ID).unwrap_or_default();
    let topic_id = get_setting(conn, SETTING_INBOUND_TOPIC_ID).filter(|v| !v.is_empty());
    let topic_title = topic_id.as_ref().and_then(|id| {
        conn.query_row(
            "SELECT title FROM forum_topics WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .ok()
    });
    let pending_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM whatsapp_inbound_messages WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    WhatsAppInboundRoutingSettings {
        enabled,
        course_id,
        topic_id,
        topic_title,
        pending_count,
    }
}

fn find_user_by_phone(conn: &Connection, from: &str) -> Option<(String, String)> {
    let country_code = get_country_code(conn);
    let target = normalize_phone(from, &country_code)?;
    let mut stmt = conn
        .prepare("SELECT id, name, phone FROM users WHERE phone IS NOT NULL AND phone != ''")
        .ok()?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    });
    let rows = rows.ok()?;
    for row in rows.flatten() {
        let (id, name, phone) = row;
        let stored = normalize_phone(&phone, &country_code).unwrap_or_else(|| {
            phone.chars().filter(|c| c.is_ascii_digit()).collect()
        });
        if stored == target {
            return Some((id, name));
        }
    }
    None
}

fn ensure_routing_topic(conn: &Connection, course_id: &str) -> Result<String, String> {
    if let Some(topic_id) = get_setting(conn, SETTING_INBOUND_TOPIC_ID).filter(|v| !v.is_empty()) {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM forum_topics WHERE id = ?1 AND course_id = ?2",
                params![topic_id, course_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if exists > 0 {
            return Ok(topic_id);
        }
    }

    let topic_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO forum_topics (id, course_id, title, author_name, created_at)
         VALUES (?1, ?2, 'WhatsApp Replies', 'ClassMate', ?3)",
        params![topic_id, course_id, now],
    )
    .map_err(|e| e.to_string())?;
    set_setting(conn, SETTING_INBOUND_TOPIC_ID, &topic_id)?;
    Ok(topic_id)
}

fn route_inbound_to_forum(conn: &Connection, inbound_id: &str) -> Result<(String, String), String> {
    let settings = read_inbound_routing_settings(conn);
    if settings.course_id.trim().is_empty() {
        return Err("Inbound routing course is not configured".into());
    }

    let (body, author_name, status): (String, Option<String>, String) = conn
        .query_row(
            "SELECT body, from_user_name, status FROM whatsapp_inbound_messages WHERE id = ?1",
            params![inbound_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Inbound message not found".to_string())?;

    if status == "routed" {
        return Err("Message already routed".into());
    }
    if status == "ignored" {
        return Err("Message was ignored".into());
    }

    let topic_id = ensure_routing_topic(conn, settings.course_id.trim())?;
    let post_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let author = author_name.unwrap_or_else(|| "WhatsApp user".into());
    let post_body = format!("📱 WhatsApp from {author}:\n\n{body}");

    conn.execute(
        "INSERT INTO forum_posts (id, topic_id, author_name, body, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![post_id, topic_id, author, post_body, now],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE whatsapp_inbound_messages
         SET status = 'routed', routed_topic_id = ?1, routed_post_id = ?2
         WHERE id = ?3",
        params![topic_id, post_id, inbound_id],
    )
    .map_err(|e| e.to_string())?;

    Ok((topic_id, post_id))
}

fn store_inbound_message(
    conn: &Connection,
    wa_message_id: &str,
    from_phone: &str,
    body: &str,
    received_at: &str,
) -> Result<Option<String>, String> {
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM whatsapp_inbound_messages WHERE wa_message_id = ?1",
            params![wa_message_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        return Ok(None);
    }

    let (from_user_id, from_user_name) = find_user_by_phone(conn, from_phone)
        .map(|(id, name)| (Some(id), Some(name)))
        .unwrap_or((None, None));

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO whatsapp_inbound_messages
         (id, wa_message_id, from_phone, from_user_id, from_user_name, body, status, received_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7)",
        params![
            id,
            wa_message_id,
            from_phone,
            from_user_id,
            from_user_name,
            body,
            received_at
        ],
    )
    .map_err(|e| e.to_string())?;

    if let Some(ref uid) = from_user_id {
        if maybe_process_unsubscribe(conn, uid, body)? {
            conn.execute(
                "UPDATE whatsapp_inbound_messages SET status = 'unsubscribe' WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
            return Ok(Some(id));
        }
    }

    let settings = read_inbound_routing_settings(conn);
    if settings.enabled && !settings.course_id.trim().is_empty() {
        let _ = route_inbound_to_forum(conn, &id);
    }

    Ok(Some(id))
}

#[tauri::command]
pub fn get_whatsapp_inbound_routing_settings(
    state: State<'_, AppState>,
) -> Result<WhatsAppInboundRoutingSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(read_inbound_routing_settings(&conn))
}

#[tauri::command]
pub fn set_whatsapp_inbound_routing_settings(
    state: State<'_, AppState>,
    input: SaveWhatsAppInboundRoutingSettingsInput,
) -> Result<WhatsAppInboundRoutingSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if input.enabled && input.course_id.trim().is_empty() {
        return Err("Select a course for inbound routing".into());
    }
    if !input.course_id.trim().is_empty() {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM courses WHERE id = ?1",
                params![input.course_id.trim()],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err("Course not found".into());
        }
    }
    set_setting(
        &conn,
        SETTING_INBOUND_ENABLED,
        if input.enabled { "true" } else { "false" },
    )?;
    set_setting(&conn, SETTING_INBOUND_COURSE_ID, input.course_id.trim())?;
    if let Some(topic_id) = input.topic_id.filter(|v| !v.trim().is_empty()) {
        set_setting(&conn, SETTING_INBOUND_TOPIC_ID, topic_id.trim())?;
    }
    Ok(read_inbound_routing_settings(&conn))
}

#[tauri::command]
pub fn list_whatsapp_inbound_messages(
    state: State<'_, AppState>,
    status: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<WhatsAppInboundMessage>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let max = limit.unwrap_or(50).clamp(1, 200);
    let sql = if status.as_ref().is_some_and(|s| !s.is_empty()) {
        "SELECT id, wa_message_id, from_phone, from_user_id, from_user_name, body, status,
                routed_topic_id, routed_post_id, received_at
         FROM whatsapp_inbound_messages
         WHERE status = ?1
         ORDER BY received_at DESC
         LIMIT ?2"
    } else {
        "SELECT id, wa_message_id, from_phone, from_user_id, from_user_name, body, status,
                routed_topic_id, routed_post_id, received_at
         FROM whatsapp_inbound_messages
         ORDER BY received_at DESC
         LIMIT ?1"
    };

    let map_row = |row: &rusqlite::Row<'_>| {
        Ok(WhatsAppInboundMessage {
            id: row.get(0)?,
            wa_message_id: row.get(1)?,
            from_phone: row.get(2)?,
            from_user_id: row.get(3)?,
            from_user_name: row.get(4)?,
            body: row.get(5)?,
            status: row.get(6)?,
            routed_topic_id: row.get(7)?,
            routed_post_id: row.get(8)?,
            received_at: row.get(9)?,
        })
    };

    let messages = if let Some(ref st) = status.filter(|s| !s.is_empty()) {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![st, max], map_row)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![max], map_row)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(messages)
}

#[tauri::command]
pub fn route_whatsapp_inbound_message(
    state: State<'_, AppState>,
    inbound_id: String,
) -> Result<WhatsAppInboundMessage, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    route_inbound_to_forum(&conn, &inbound_id)?;
    conn.query_row(
        "SELECT id, wa_message_id, from_phone, from_user_id, from_user_name, body, status,
                routed_topic_id, routed_post_id, received_at
         FROM whatsapp_inbound_messages WHERE id = ?1",
        params![inbound_id],
        |row| {
            Ok(WhatsAppInboundMessage {
                id: row.get(0)?,
                wa_message_id: row.get(1)?,
                from_phone: row.get(2)?,
                from_user_id: row.get(3)?,
                from_user_name: row.get(4)?,
                body: row.get(5)?,
                status: row.get(6)?,
                routed_topic_id: row.get(7)?,
                routed_post_id: row.get(8)?,
                received_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ignore_whatsapp_inbound_message(
    state: State<'_, AppState>,
    inbound_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let n = conn
        .execute(
            "UPDATE whatsapp_inbound_messages SET status = 'ignored'
             WHERE id = ?1 AND status = 'pending'",
            params![inbound_id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("Message not found or already processed".into());
    }
    Ok(())
}

fn parse_scheduled_at(raw: &str) -> Result<String, String> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
        .map_err(|_| "scheduled_at must be a valid ISO/RFC3339 datetime".into())
}

fn map_scheduled_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WhatsAppScheduledBroadcast> {
    Ok(WhatsAppScheduledBroadcast {
        id: row.get(0)?,
        group_id: row.get(1)?,
        group_name: row.get(2)?,
        broadcast_kind: row.get(3)?,
        kind: row.get(4)?,
        assignment_id: row.get(5)?,
        announcement_id: row.get(6)?,
        custom_message: row.get(7)?,
        scheduled_at: row.get(8)?,
        status: row.get(9)?,
        sent_at: row.get(10)?,
        result_sent: row.get(11)?,
        result_failed: row.get(12)?,
        result_skipped: row.get(13)?,
        error: row.get(14)?,
        created_at: row.get(15)?,
    })
}

fn run_scheduled_broadcast(conn: &Connection, id: &str) -> Result<WhatsAppBroadcastResult, String> {
    let row = conn.query_row(
        "SELECT id, group_id, broadcast_kind, kind, assignment_id, announcement_id, custom_message
         FROM whatsapp_scheduled_broadcasts
         WHERE id = ?1 AND status = 'pending'",
        params![id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        },
    );

    let (
        _id,
        group_id,
        broadcast_kind,
        kind,
        assignment_id,
        announcement_id,
        custom_message,
    ) = match row {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err("Scheduled broadcast not found or already processed".into())
        }
        Err(e) => return Err(e.to_string()),
    };

    let result = if broadcast_kind == "template" {
        let assignment_id = assignment_id.ok_or("Template broadcast requires assignment_id")?;
        execute_template_broadcast(
            conn,
            &SendWhatsAppTemplateBroadcastInput {
                group_id: group_id.clone(),
                assignment_id,
            },
        )
    } else {
        let kind = kind.ok_or("Text broadcast requires kind")?;
        execute_text_broadcast(
            conn,
            None,
            &SendWhatsAppBroadcastInput {
                kind,
                group_id,
                assignment_id,
                announcement_id,
                custom_message,
            },
        )
    };

    let sent_at = Utc::now().to_rfc3339();
    match &result {
        Ok(stats) => {
            let summary = if stats.errors.is_empty() {
                None
            } else {
                Some(stats.errors.join("; "))
            };
            conn.execute(
                "UPDATE whatsapp_scheduled_broadcasts
                 SET status = 'sent', sent_at = ?1, result_sent = ?2, result_failed = ?3,
                     result_skipped = ?4, error = ?5
                 WHERE id = ?6",
                params![
                    sent_at,
                    stats.sent,
                    stats.failed,
                    stats.skipped,
                    summary,
                    id
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        Err(err) => {
            conn.execute(
                "UPDATE whatsapp_scheduled_broadcasts
                 SET status = 'failed', sent_at = ?1, error = ?2
                 WHERE id = ?3",
                params![sent_at, err, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    result
}

pub fn maybe_run_scheduled_whatsapp_broadcasts(conn: &Connection) -> Result<WhatsAppScheduledRunResult, String> {
    if load_business_config(conn).is_err() {
        return Ok(WhatsAppScheduledRunResult {
            processed: 0,
            sent: 0,
            failed: 0,
        });
    }

    let now = Utc::now().to_rfc3339();
    let mut stmt = conn
        .prepare(
            "SELECT id FROM whatsapp_scheduled_broadcasts
             WHERE status = 'pending' AND scheduled_at <= ?1
             ORDER BY scheduled_at ASC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;
    let ids: Vec<String> = stmt
        .query_map(params![now], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut processed = 0i64;
    let mut sent = 0i64;
    let mut failed = 0i64;

    for id in ids {
        match run_scheduled_broadcast(conn, &id) {
            Ok(stats) => {
                processed += 1;
                if stats.failed == 0 && stats.sent > 0 {
                    sent += 1;
                } else if stats.sent == 0 && stats.failed > 0 {
                    failed += 1;
                } else if stats.failed > 0 {
                    failed += 1;
                } else {
                    sent += 1;
                }
            }
            Err(_) => {
                processed += 1;
                failed += 1;
            }
        }
    }

    Ok(WhatsAppScheduledRunResult {
        processed,
        sent,
        failed,
    })
}

#[tauri::command]
pub fn create_whatsapp_scheduled_broadcast(
    state: State<'_, AppState>,
    input: CreateWhatsAppScheduledBroadcastInput,
) -> Result<WhatsAppScheduledBroadcast, String> {
    if !["text", "template"].contains(&input.broadcast_kind.as_str()) {
        return Err("broadcast_kind must be text or template".into());
    }
    if input.broadcast_kind == "template" && input.assignment_id.as_ref().is_none_or(|v| v.is_empty()) {
        return Err("Template broadcasts require assignment_id".into());
    }
    if input.broadcast_kind == "text" && input.kind.as_ref().is_none_or(|v| v.is_empty()) {
        return Err("Text broadcasts require kind".into());
    }

    let scheduled_at = parse_scheduled_at(input.scheduled_at.trim())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM whatsapp_groups WHERE id = ?1",
            params![input.group_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if exists == 0 {
        return Err("Contact group not found".into());
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO whatsapp_scheduled_broadcasts
         (id, group_id, broadcast_kind, kind, assignment_id, announcement_id, custom_message,
          scheduled_at, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', ?9)",
        params![
            id,
            input.group_id,
            input.broadcast_kind,
            input.kind,
            input.assignment_id,
            input.announcement_id,
            input.custom_message,
            scheduled_at,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT s.id, s.group_id, g.name, s.broadcast_kind, s.kind, s.assignment_id,
                s.announcement_id, s.custom_message, s.scheduled_at, s.status, s.sent_at,
                s.result_sent, s.result_failed, s.result_skipped, s.error, s.created_at
         FROM whatsapp_scheduled_broadcasts s
         LEFT JOIN whatsapp_groups g ON g.id = s.group_id
         WHERE s.id = ?1",
        params![id],
        map_scheduled_row,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_whatsapp_scheduled_broadcasts(
    state: State<'_, AppState>,
    group_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<WhatsAppScheduledBroadcast>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let max = limit.unwrap_or(50).clamp(1, 200);
    let sql = if group_id.is_some() {
        "SELECT s.id, s.group_id, g.name, s.broadcast_kind, s.kind, s.assignment_id,
                s.announcement_id, s.custom_message, s.scheduled_at, s.status, s.sent_at,
                s.result_sent, s.result_failed, s.result_skipped, s.error, s.created_at
         FROM whatsapp_scheduled_broadcasts s
         LEFT JOIN whatsapp_groups g ON g.id = s.group_id
         WHERE s.group_id = ?1
         ORDER BY s.scheduled_at DESC
         LIMIT ?2"
    } else {
        "SELECT s.id, s.group_id, g.name, s.broadcast_kind, s.kind, s.assignment_id,
                s.announcement_id, s.custom_message, s.scheduled_at, s.status, s.sent_at,
                s.result_sent, s.result_failed, s.result_skipped, s.error, s.created_at
         FROM whatsapp_scheduled_broadcasts s
         LEFT JOIN whatsapp_groups g ON g.id = s.group_id
         ORDER BY s.scheduled_at DESC
         LIMIT ?1"
    };

    let rows = if let Some(ref gid) = group_id {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map(params![gid, max], map_scheduled_row)
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map(params![max], map_scheduled_row)
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(rows)
}

#[tauri::command]
pub fn cancel_whatsapp_scheduled_broadcast(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let n = conn
        .execute(
            "UPDATE whatsapp_scheduled_broadcasts SET status = 'cancelled'
             WHERE id = ?1 AND status = 'pending'",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("Scheduled broadcast not found or not pending".into());
    }
    Ok(())
}

#[tauri::command]
pub fn run_due_whatsapp_scheduled_broadcasts(
    state: State<'_, AppState>,
) -> Result<WhatsAppScheduledRunResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    maybe_run_scheduled_whatsapp_broadcasts(&conn)
}

#[derive(Deserialize)]
struct WebhookPayload {
    #[serde(default)]
    entry: Vec<WebhookEntry>,
}

#[derive(Deserialize)]
struct WebhookEntry {
    #[serde(default)]
    changes: Vec<WebhookChange>,
}

#[derive(Deserialize)]
struct WebhookChange {
    #[serde(default)]
    field: String,
    #[serde(default)]
    value: WebhookValue,
}

#[derive(Default, Deserialize)]
struct WebhookValue {
    #[serde(default)]
    statuses: Vec<WebhookStatus>,
    #[serde(default)]
    messages: Vec<WebhookInboundMessage>,
    #[serde(default)]
    groups: Vec<WebhookGroupEntry>,
    #[serde(default)]
    group_id: Option<String>,
    #[serde(default)]
    invite_link: Option<String>,
    #[serde(default)]
    event: Option<String>,
}

#[derive(Default, Deserialize)]
struct WebhookGroupEntry {
    #[serde(default)]
    group_id: Option<String>,
    #[serde(default, rename = "type")]
    event_type: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    invite_link: Option<String>,
    #[serde(default)]
    wa_id: Option<String>,
    #[serde(default)]
    join_request_id: Option<String>,
    #[serde(default)]
    added_participants: Vec<WebhookParticipantRef>,
    #[serde(default)]
    removed_participants: Vec<WebhookParticipantRef>,
    #[serde(default)]
    group_subject: Option<WebhookGroupSettingText>,
    #[serde(default)]
    group_description: Option<WebhookGroupSettingText>,
    #[serde(default)]
    profile_picture: Option<WebhookGroupProfilePicture>,
}

#[derive(Default, Deserialize)]
struct WebhookGroupSettingText {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    update_successful: Option<bool>,
    #[serde(default)]
    errors: Vec<WebhookSettingError>,
}

#[derive(Default, Deserialize)]
struct WebhookGroupProfilePicture {
    #[serde(default)]
    mime_type: Option<String>,
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default)]
    update_successful: Option<bool>,
    #[serde(default)]
    errors: Vec<WebhookSettingError>,
}

#[derive(Default, Deserialize)]
struct WebhookSettingError {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Default, Deserialize)]
struct WebhookParticipantRef {
    #[serde(default)]
    wa_id: Option<String>,
    #[serde(default)]
    input: Option<String>,
}

#[derive(Deserialize)]
struct WebhookInboundMessage {
    from: String,
    id: String,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    text: Option<WebhookTextBody>,
}

#[derive(Deserialize)]
struct WebhookTextBody {
    body: String,
}

#[derive(Deserialize)]
struct WebhookStatus {
    id: String,
    status: String,
    #[serde(default)]
    timestamp: Option<String>,
}

pub fn verify_webhook_token(conn: &Connection, token: &str) -> bool {
    get_setting(conn, SETTING_WEBHOOK_VERIFY_TOKEN)
        .map(|expected| !expected.is_empty() && expected == token)
        .unwrap_or(false)
}

pub fn apply_webhook_payload(conn: &Connection, body: &str) -> Result<i64, String> {
    let payload: WebhookPayload = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let mut updated = 0i64;
    for entry in payload.entry {
        for change in entry.changes {
            if change.field == "group_lifecycle_update" {
                updated += process_group_lifecycle_update(conn, &change.value)?;
            }
            if change.field == "group_participants_update" {
                updated += process_group_participants_update(conn, &change.value)?;
            }
            if change.field == "group_settings_update" {
                updated += process_group_settings_update(conn, &change.value)?;
            }
            if change.field == "group_status_update" {
                updated += process_group_status_update(conn, &change.value)?;
            }
            for status in change.value.statuses {
                updated += update_message_status(conn, &status)?;
            }
            for message in change.value.messages {
                updated += process_inbound_webhook_message(conn, &message)?;
            }
        }
    }
    Ok(updated)
}

fn process_group_lifecycle_update(
    conn: &Connection,
    value: &WebhookValue,
) -> Result<i64, String> {
    let mut updated = 0i64;
    let now = Utc::now().to_rfc3339();

    if !value.groups.is_empty() {
        for group in &value.groups {
            let Some(external_group_id) = group.group_id.as_ref().filter(|s| !s.is_empty()) else {
                continue;
            };
            if let Some(invite_link) = group.invite_link.as_ref().filter(|s| !s.is_empty()) {
                let n = conn
                    .execute(
                        "UPDATE whatsapp_group_links
                         SET invite_link = ?1, native_status = 'active', creation_error = NULL, linked_at = ?2
                         WHERE external_group_id = ?3",
                        params![invite_link, now, external_group_id],
                    )
                    .map_err(|e| e.to_string())?;
                updated += n as i64;
            }
        }
        return Ok(updated);
    }

    let Some(external_group_id) = value.group_id.as_ref().filter(|s| !s.is_empty()) else {
        return Ok(0);
    };
    let Some(invite_link) = value.invite_link.as_ref().filter(|s| !s.is_empty()) else {
        return Ok(0);
    };
    let n = conn
        .execute(
            "UPDATE whatsapp_group_links
             SET invite_link = ?1, native_status = 'active', creation_error = NULL, linked_at = ?2
             WHERE external_group_id = ?3",
            params![invite_link, now, external_group_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(n as i64)
}

fn process_group_participants_update(
    conn: &Connection,
    value: &WebhookValue,
) -> Result<i64, String> {
    let mut updated = 0i64;
    for group in &value.groups {
        let Some(external_group_id) = group.group_id.as_ref().filter(|s| !s.is_empty()) else {
            continue;
        };
        let classmate_group_id = lookup_classmate_group_id(conn, external_group_id);
        let event_type = group
            .event_type
            .as_deref()
            .unwrap_or("group_participants_update");
        let reason = group.reason.as_deref();

        for item in &group.added_participants {
            if let Some(wa_id) = participant_wa_id(item) {
                insert_participant_event(
                    conn,
                    classmate_group_id.as_deref(),
                    external_group_id,
                    event_type,
                    Some("added"),
                    Some(&wa_id),
                    reason,
                    None,
                )?;
                updated += 1;
            }
        }
        for item in &group.removed_participants {
            if let Some(wa_id) = participant_wa_id(item) {
                insert_participant_event(
                    conn,
                    classmate_group_id.as_deref(),
                    external_group_id,
                    event_type,
                    Some("removed"),
                    Some(&wa_id),
                    reason,
                    None,
                )?;
                updated += 1;
            }
        }
        if let Some(wa_id) = group.wa_id.as_ref().filter(|s| !s.is_empty()) {
            let direction = if event_type.contains("revoked") {
                "removed"
            } else if event_type.contains("request") {
                "request"
            } else {
                "updated"
            };
            insert_participant_event(
                conn,
                classmate_group_id.as_deref(),
                external_group_id,
                event_type,
                Some(direction),
                Some(wa_id),
                reason,
                group.join_request_id.as_deref(),
            )?;
            updated += 1;
        }

        if lookup_classmate_group_id(conn, external_group_id).is_some() {
            let _ = refresh_cached_group_participants(conn, external_group_id);
            updated += 1;
        }
    }
    Ok(updated)
}

fn webhook_setting_error_summary(setting: &WebhookGroupSettingText) -> Option<String> {
    setting.errors.first().and_then(|e| {
        e.message
            .as_deref()
            .or(e.title.as_deref())
            .map(|s| s.to_string())
    })
}

fn webhook_profile_error_summary(picture: &WebhookGroupProfilePicture) -> Option<String> {
    picture.errors.first().and_then(|e| {
        e.message
            .as_deref()
            .or(e.title.as_deref())
            .map(|s| s.to_string())
    })
}

fn apply_successful_group_settings(
    conn: &Connection,
    external_group_id: &str,
    subject: Option<&WebhookGroupSettingText>,
    description: Option<&WebhookGroupSettingText>,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    if let Some(subject) = subject {
        if subject.update_successful == Some(true) {
            if let Some(text) = subject.text.as_ref().filter(|s| !s.is_empty()) {
                conn.execute(
                    "UPDATE whatsapp_group_links
                     SET external_name = ?1, settings_updated_at = ?2
                     WHERE external_group_id = ?3",
                    params![text, now, external_group_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    if let Some(description) = description {
        if description.update_successful == Some(true) {
            if let Some(text) = description.text.as_ref().filter(|s| !s.is_empty()) {
                conn.execute(
                    "UPDATE whatsapp_group_links
                     SET group_description = ?1, settings_updated_at = ?2
                     WHERE external_group_id = ?3",
                    params![text, now, external_group_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

fn process_group_settings_update(conn: &Connection, value: &WebhookValue) -> Result<i64, String> {
    let mut updated = 0i64;
    for group in &value.groups {
        let Some(external_group_id) = group.group_id.as_ref().filter(|s| !s.is_empty()) else {
            continue;
        };
        let classmate_group_id = lookup_classmate_group_id(conn, external_group_id);
        let event_type = group
            .event_type
            .as_deref()
            .unwrap_or("group_settings_update");

        if let Some(subject) = group.group_subject.as_ref() {
            insert_settings_event(
                conn,
                classmate_group_id.as_deref(),
                external_group_id,
                event_type,
                Some("subject"),
                subject.text.as_deref(),
                subject.update_successful,
                webhook_setting_error_summary(subject).as_deref(),
            )?;
            updated += 1;
        }
        if let Some(description) = group.group_description.as_ref() {
            insert_settings_event(
                conn,
                classmate_group_id.as_deref(),
                external_group_id,
                event_type,
                Some("description"),
                description.text.as_deref(),
                description.update_successful,
                webhook_setting_error_summary(description).as_deref(),
            )?;
            updated += 1;
        }
        if let Some(picture) = group.profile_picture.as_ref() {
            insert_settings_event(
                conn,
                classmate_group_id.as_deref(),
                external_group_id,
                event_type,
                Some("profile_picture"),
                picture.sha256.as_deref(),
                picture.update_successful,
                webhook_profile_error_summary(picture).as_deref(),
            )?;
            updated += 1;
        }

        apply_successful_group_settings(
            conn,
            external_group_id,
            group.group_subject.as_ref(),
            group.group_description.as_ref(),
        )?;
    }
    Ok(updated)
}

fn process_group_status_update(conn: &Connection, value: &WebhookValue) -> Result<i64, String> {
    let mut updated = 0i64;
    for group in &value.groups {
        let Some(external_group_id) = group.group_id.as_ref().filter(|s| !s.is_empty()) else {
            continue;
        };
        let classmate_group_id = lookup_classmate_group_id(conn, external_group_id);
        let event_type = group
            .event_type
            .as_deref()
            .unwrap_or("group_status_update");

        insert_settings_event(
            conn,
            classmate_group_id.as_deref(),
            external_group_id,
            event_type,
            Some("status"),
            Some(event_type),
            None,
            None,
        )?;
        updated += 1;

        let native_status = match event_type {
            "group_suspend" => Some("suspended"),
            "group_suspend_cleared" => Some("active"),
            _ => None,
        };
        if let Some(status) = native_status {
            let n = conn
                .execute(
                    "UPDATE whatsapp_group_links SET native_status = ?1 WHERE external_group_id = ?2",
                    params![status, external_group_id],
                )
                .map_err(|e| e.to_string())?;
            updated += n as i64;
        }
    }
    Ok(updated)
}

fn process_inbound_webhook_message(
    conn: &Connection,
    message: &WebhookInboundMessage,
) -> Result<i64, String> {
    if message.message_type != "text" {
        return Ok(0);
    }
    let Some(text) = message.text.as_ref() else {
        return Ok(0);
    };
    if text.body.trim().is_empty() {
        return Ok(0);
    }
    let received_at = message
        .timestamp
        .as_ref()
        .and_then(|t| {
            t.parse::<i64>()
                .ok()
                .and_then(|secs| chrono::DateTime::from_timestamp(secs, 0))
                .map(|dt| dt.to_rfc3339())
        })
        .unwrap_or_else(|| Utc::now().to_rfc3339());

    match store_inbound_message(conn, &message.id, &message.from, &text.body, &received_at)? {
        Some(_) => Ok(1),
        None => Ok(0),
    }
}

fn update_message_status(conn: &Connection, status: &WebhookStatus) -> Result<i64, String> {
    let ts = status
        .timestamp
        .as_ref()
        .and_then(|t| {
            t.parse::<i64>()
                .ok()
                .and_then(|secs| chrono::DateTime::from_timestamp(secs, 0))
                .map(|dt| dt.to_rfc3339())
        })
        .unwrap_or_else(|| Utc::now().to_rfc3339());

    let (col, mapped_status) = match status.status.as_str() {
        "sent" => ("sent_at", "sent"),
        "delivered" => ("delivered_at", "delivered"),
        "read" => ("read_at", "read"),
        "failed" => ("error", "failed"),
        _ => return Ok(0),
    };

    if status.status == "failed" {
        let n = conn
            .execute(
                "UPDATE whatsapp_outbound_messages
                 SET status = 'failed', error = COALESCE(error, 'Delivery failed')
                 WHERE wa_message_id = ?1",
                params![status.id],
            )
            .map_err(|e| e.to_string())?;
        return Ok(n as i64);
    }

    let sql = format!(
        "UPDATE whatsapp_outbound_messages
         SET status = ?1, {col} = ?2
         WHERE wa_message_id = ?3"
    );
    let n = conn
        .execute(&sql, params![mapped_status, ts, status.id])
        .map_err(|e| e.to_string())?;
    Ok(n as i64)
}
