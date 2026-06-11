use crate::models::{
    CreateWhatsAppGroupInput, HubStatus, UpdateUserPhoneInput, WhatsAppGroup,
    WhatsAppGroupMember, WhatsAppShareInput, WhatsAppSharePlan, WhatsAppShareRecipient,
};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

pub(crate) fn get_country_code(conn: &rusqlite::Connection) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'whatsapp_country_code'",
        [],
        |row| row.get(0),
    )
    .unwrap_or_else(|_| "1".into())
}

pub(crate) fn normalize_phone(raw: &str, country_code: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 7 {
        return None;
    }
    if digits.len() >= 10 && !raw.trim_start().starts_with('+') && country_code != "0" {
        Some(format!("{country_code}{digits}"))
    } else {
        Some(digits)
    }
}

fn url_encode_message(message: &str) -> String {
    message
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => (b as char).to_string(),
            b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            b' ' => "+".into(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn wa_me_url(phone_digits: &str, message: &str) -> String {
    format!(
        "https://wa.me/{phone_digits}?text={}",
        url_encode_message(message)
    )
}

fn build_assignment_message(
    course_title: &str,
    title: &str,
    description: Option<&str>,
    due_at: Option<&str>,
) -> String {
    let mut lines = vec![
        format!("📚 {course_title}"),
        format!("📝 Assignment: {title}"),
    ];
    if let Some(due) = due_at {
        lines.push(format!("Due: {due}"));
    }
    if let Some(desc) = description.filter(|d| !d.trim().is_empty()) {
        lines.push(String::new());
        lines.push(desc.to_string());
    }
    lines.push(String::new());
    lines.push("— ClassMate".into());
    lines.join("\n")
}

fn build_announcement_message(title: &str, body: &str, course_title: Option<&str>) -> String {
    let mut lines = vec![format!("📢 {title}")];
    if let Some(course) = course_title {
        lines.push(format!("Course: {course}"));
    }
    lines.push(String::new());
    lines.push(body.to_string());
    lines.push(String::new());
    lines.push("— ClassMate".into());
    lines.join("\n")
}

fn build_hub_message(course_title: &str, pin: &str, join_url: Option<&str>) -> String {
    let mut lines = vec![
        format!("🎓 Live class: {course_title}"),
        format!("PIN: {pin}"),
    ];
    if let Some(url) = join_url {
        lines.push(format!("Join: {url}"));
    }
    lines.push(String::new());
    lines.push("— ClassMate".into());
    lines.join("\n")
}

pub fn compose_share_message(
    conn: &rusqlite::Connection,
    hub_status: Option<HubStatus>,
    input: &WhatsAppShareInput,
) -> Result<String, String> {
    match input.kind.as_str() {
        "assignment" | "task" => {
            let assignment_id = input
                .assignment_id
                .as_ref()
                .ok_or("assignment_id required")?;
            let (course_title, title, description, due_at): (
                String,
                String,
                Option<String>,
                Option<String>,
            ) = conn
                .query_row(
                    "SELECT c.title, a.title, a.description, a.due_at
                     FROM assignments a
                     JOIN courses c ON c.id = a.course_id
                     WHERE a.id = ?1",
                    params![assignment_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .map_err(|_| "Assignment not found".to_string())?;
            Ok(build_assignment_message(
                &course_title,
                &title,
                description.as_deref(),
                due_at.as_deref(),
            ))
        }
        "announcement" => {
            let announcement_id = input
                .announcement_id
                .as_ref()
                .ok_or("announcement_id required")?;
            let (title, body, course_title): (String, String, Option<String>) = conn
                .query_row(
                    "SELECT a.title, a.body, c.title
                     FROM announcements a
                     LEFT JOIN courses c ON c.id = a.course_id
                     WHERE a.id = ?1",
                    params![announcement_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|_| "Announcement not found".to_string())?;
            Ok(build_announcement_message(
                &title,
                &body,
                course_title.as_deref(),
            ))
        }
        "hub" => {
            let hub_status = hub_status.ok_or("No live class hub session")?;
            if !hub_status.running {
                return Err("No live class hub session".into());
            }
            let course_title = hub_status
                .course_title
                .unwrap_or_else(|| "Class".into());
            let pin = hub_status.pin.unwrap_or_default();
            Ok(build_hub_message(
                &course_title,
                &pin,
                hub_status.join_url.as_deref(),
            ))
        }
        _ => input
            .custom_message
            .clone()
            .ok_or_else(|| "Unknown share kind or missing custom_message".into()),
    }
}

#[tauri::command]
pub fn list_whatsapp_groups(
    state: State<'_, AppState>,
    course_id: Option<String>,
) -> Result<Vec<WhatsAppGroup>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sql = if course_id.is_some() {
        "SELECT g.id, g.course_id, c.title, g.name, g.kind,
                (SELECT COUNT(*) FROM whatsapp_group_members m WHERE m.group_id = g.id),
                g.created_at, l.invite_link, l.external_name, l.external_group_id, l.native_status, l.creation_error, l.group_description
         FROM whatsapp_groups g
         JOIN courses c ON c.id = g.course_id
         LEFT JOIN whatsapp_group_links l ON l.group_id = g.id
         WHERE g.course_id = ?1
         ORDER BY g.name COLLATE NOCASE"
    } else {
        "SELECT g.id, g.course_id, c.title, g.name, g.kind,
                (SELECT COUNT(*) FROM whatsapp_group_members m WHERE m.group_id = g.id),
                g.created_at, l.invite_link, l.external_name, l.external_group_id, l.native_status, l.creation_error, l.group_description
         FROM whatsapp_groups g
         JOIN courses c ON c.id = g.course_id
         LEFT JOIN whatsapp_group_links l ON l.group_id = g.id
         ORDER BY c.title COLLATE NOCASE, g.name COLLATE NOCASE"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let map_row = |row: &rusqlite::Row<'_>| {
        Ok(WhatsAppGroup {
            id: row.get(0)?,
            course_id: row.get(1)?,
            course_title: row.get(2)?,
            name: row.get(3)?,
            kind: row.get(4)?,
            member_count: row.get(5)?,
            created_at: row.get(6)?,
            invite_link: row.get(7)?,
            external_name: row.get(8)?,
            external_group_id: row.get(9)?,
            native_status: row.get(10)?,
            creation_error: row.get(11)?,
            group_description: row.get(12)?,
        })
    };

    let groups = if let Some(ref cid) = course_id {
        stmt.query_map(params![cid], map_row)
    } else {
        stmt.query_map([], map_row)
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(groups)
}

#[tauri::command]
pub fn create_whatsapp_group(
    state: State<'_, AppState>,
    input: CreateWhatsAppGroupInput,
) -> Result<WhatsAppGroup, String> {
    if !matches!(
        input.kind.as_str(),
        "students" | "teachers" | "custom"
    ) {
        return Err("Invalid group kind".into());
    }

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let course_title: String = conn
        .query_row(
            "SELECT title FROM courses WHERE id = ?1",
            params![input.course_id],
            |row| row.get(0),
        )
        .map_err(|_| "Course not found".to_string())?;

    conn.execute(
        "INSERT INTO whatsapp_groups (id, course_id, name, kind, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, input.course_id, input.name, input.kind, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(WhatsAppGroup {
        id,
        course_id: input.course_id,
        course_title,
        name: input.name,
        kind: input.kind,
        member_count: 0,
        created_at: now,
        invite_link: None,
        external_name: None,
        external_group_id: None,
        native_status: None,
        creation_error: None,
        group_description: None,
    })
}

#[tauri::command]
pub fn delete_whatsapp_group(state: State<'_, AppState>, group_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM whatsapp_groups WHERE id = ?1",
        params![group_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_whatsapp_group_members(
    state: State<'_, AppState>,
    group_id: String,
) -> Result<Vec<WhatsAppGroupMember>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.name, u.phone, u.role, COALESCE(c.opted_in, 0)
             FROM whatsapp_group_members m
             JOIN users u ON u.id = m.user_id
             LEFT JOIN whatsapp_consent c ON c.user_id = u.id
             WHERE m.group_id = ?1
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map(params![group_id], |row| {
            Ok(WhatsAppGroupMember {
                user_id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                role: row.get(3)?,
                opted_in: row.get::<_, i64>(4)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(members)
}

#[tauri::command]
pub fn add_whatsapp_group_member(
    state: State<'_, AppState>,
    group_id: String,
    user_id: String,
) -> Result<(), String> {
    let id = Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO whatsapp_group_members (id, group_id, user_id) VALUES (?1, ?2, ?3)",
        params![id, group_id, user_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_whatsapp_group_member(
    state: State<'_, AppState>,
    group_id: String,
    user_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM whatsapp_group_members WHERE group_id = ?1 AND user_id = ?2",
        params![group_id, user_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn sync_whatsapp_group_members(
    state: State<'_, AppState>,
    group_id: String,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (course_id, kind): (String, String) = conn
        .query_row(
            "SELECT course_id, kind FROM whatsapp_groups WHERE id = ?1",
            params![group_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Group not found".to_string())?;

    if kind == "custom" {
        return Err("Custom groups must be managed manually".into());
    }

    conn.execute(
        "DELETE FROM whatsapp_group_members WHERE group_id = ?1",
        params![group_id],
    )
    .map_err(|e| e.to_string())?;

    let mut added = 0i64;
    if kind == "students" {
        let mut stmt = conn
            .prepare(
                "SELECT u.id FROM enrollments e
                 JOIN users u ON u.id = e.student_id
                 WHERE e.course_id = ?1 AND e.status = 'active'",
            )
            .map_err(|e| e.to_string())?;
        let ids: Vec<String> = stmt
            .query_map(params![course_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for uid in ids {
            let mid = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO whatsapp_group_members (id, group_id, user_id) VALUES (?1, ?2, ?3)",
                params![mid, group_id, uid],
            )
            .map_err(|e| e.to_string())?;
            added += 1;
        }
    } else if kind == "teachers" {
        let teacher_id: Option<String> = conn
            .query_row(
                "SELECT teacher_id FROM courses WHERE id = ?1",
                params![course_id],
                |row| row.get(0),
            )
            .ok()
            .flatten();
        let ids: Vec<String> = if let Some(tid) = teacher_id {
            vec![tid]
        } else {
            let mut stmt = conn
                .prepare("SELECT id FROM users WHERE role = 'teacher'")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };
        for uid in ids {
            let mid = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT OR IGNORE INTO whatsapp_group_members (id, group_id, user_id) VALUES (?1, ?2, ?3)",
                params![mid, group_id, uid],
            )
            .map_err(|e| e.to_string())?;
            added += 1;
        }
    }

    Ok(added)
}

#[tauri::command]
pub fn update_user_phone(
    state: State<'_, AppState>,
    input: UpdateUserPhoneInput,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let phone = input.phone.filter(|p| !p.trim().is_empty());
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE users SET phone = ?1, updated_at = ?2 WHERE id = ?3",
        params![phone, now, input.user_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn build_whatsapp_share(
    state: State<'_, AppState>,
    input: WhatsAppShareInput,
) -> Result<WhatsAppSharePlan, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let country_code = get_country_code(&conn);
    let hub_status = state.hub.lock().map_err(|e| e.to_string())?.status();
    let message = compose_share_message(&conn, Some(hub_status), &input)?;

    let mut member_rows: Vec<(String, String, Option<String>)> = Vec::new();
    if let Some(ref group_id) = input.group_id {
        let mut stmt = conn
            .prepare(
                "SELECT u.id, u.name, u.phone
                 FROM whatsapp_group_members m
                 JOIN users u ON u.id = m.user_id
                 WHERE m.group_id = ?1
                 ORDER BY u.name COLLATE NOCASE",
            )
            .map_err(|e| e.to_string())?;
        member_rows = stmt
            .query_map(params![group_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
    }

    let mut recipients = Vec::new();
    let mut missing_phones = Vec::new();
    for (user_id, name, phone) in member_rows {
        let wa_url = phone
            .as_deref()
            .and_then(|p| normalize_phone(p, &country_code))
            .map(|digits| wa_me_url(&digits, &message));
        if wa_url.is_none() {
            missing_phones.push(name.clone());
        }
        recipients.push(WhatsAppShareRecipient {
            user_id,
            name,
            phone,
            wa_url,
        });
    }

    Ok(WhatsAppSharePlan {
        message: message.clone(),
        group_paste: message,
        recipients,
        missing_phones,
    })
}
