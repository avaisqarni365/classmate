use crate::commands::tenancy::{active_school_id_or_resolve, require_org_admin, require_user};
use crate::models::{
    CashbookEntry, CashbookIntegrationTest, CashbookSettings, CashbookSummary,
    CreateCashbookEntryInput, SaveCashbookSettingsInput,
};
use crate::AppState;
use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;
use uuid::Uuid;

const SETTING_CURRENCY: &str = "cashbook_currency";
const SETTING_INVOICE_NINJA_URL: &str = "cashbook_invoice_ninja_url";
const SETTING_INVOICE_NINJA_TOKEN: &str = "cashbook_invoice_ninja_token";

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

fn require_admin(state: &State<'_, AppState>) -> Result<(), String> {
    let user = require_user(state)?;
    require_org_admin(&user)
}

fn read_settings(conn: &Connection) -> CashbookSettings {
    let currency = get_setting(conn, SETTING_CURRENCY)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "USD".into());
    let invoice_ninja_url = get_setting(conn, SETTING_INVOICE_NINJA_URL).unwrap_or_default();
    let invoice_ninja_token_set = get_setting(conn, SETTING_INVOICE_NINJA_TOKEN)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    CashbookSettings {
        currency,
        invoice_ninja_configured: !invoice_ninja_url.trim().is_empty() && invoice_ninja_token_set,
        invoice_ninja_url,
    }
}

fn validate_entry(input: &CreateCashbookEntryInput) -> Result<(), String> {
    if !matches!(input.direction.as_str(), "income" | "expense") {
        return Err("Invalid direction".into());
    }
    if !matches!(
        input.category.as_str(),
        "student_fee" | "other_income" | "teacher_salary" | "other_expense"
    ) {
        return Err("Invalid category".into());
    }
    if input.amount <= 0.0 {
        return Err("Amount must be greater than zero".into());
    }
    if !matches!(
        input.payment_method.as_str(),
        "cash" | "bank" | "cheque" | "online"
    ) {
        return Err("Invalid payment method".into());
    }
    Ok(())
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<CashbookEntry> {
    Ok(CashbookEntry {
        id: row.get(0)?,
        school_id: row.get(1)?,
        direction: row.get(2)?,
        category: row.get(3)?,
        amount: row.get(4)?,
        currency: row.get(5)?,
        description: row.get(6)?,
        user_id: row.get(7)?,
        user_name: row.get(8)?,
        course_id: row.get(9)?,
        course_title: row.get(10)?,
        payment_method: row.get(11)?,
        reference: row.get(12)?,
        entry_date: row.get(13)?,
        created_by: row.get(14)?,
        created_by_name: row.get(15)?,
        created_at: row.get(16)?,
    })
}

#[tauri::command]
pub fn get_cashbook_settings(state: State<'_, AppState>) -> Result<CashbookSettings, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(read_settings(&conn))
}

#[tauri::command]
pub fn save_cashbook_settings(
    state: State<'_, AppState>,
    input: SaveCashbookSettingsInput,
) -> Result<CashbookSettings, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        SETTING_CURRENCY,
        input.currency.trim().to_uppercase().as_str(),
    )?;
    if let Some(url) = input.invoice_ninja_url.as_ref() {
        set_setting(&conn, SETTING_INVOICE_NINJA_URL, url.trim())?;
    }
    if let Some(token) = input.invoice_ninja_token.as_ref() {
        if !token.trim().is_empty() {
            set_setting(&conn, SETTING_INVOICE_NINJA_TOKEN, token.trim())?;
        }
    }
    Ok(read_settings(&conn))
}

#[tauri::command]
pub fn test_invoice_ninja_connection(
    state: State<'_, AppState>,
) -> Result<CashbookIntegrationTest, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let url = get_setting(&conn, SETTING_INVOICE_NINJA_URL)
        .filter(|v| !v.trim().is_empty())
        .ok_or("Invoice Ninja URL is not configured")?;
    let token = get_setting(&conn, SETTING_INVOICE_NINJA_TOKEN)
        .filter(|v| !v.trim().is_empty())
        .ok_or("Invoice Ninja API token is missing")?;
    let base = url.trim().trim_end_matches('/');
    let ping_url = format!("{base}/api/v1/clients?per_page=1");
    let response = ureq::get(&ping_url)
        .set("X-API-TOKEN", &token)
        .set("X-Requested-With", "XMLHttpRequest")
        .call()
        .map_err(|e| e.to_string())?;
    let status = response.status();
    if status >= 400 {
        let text = response.into_string().unwrap_or_default();
        return Ok(CashbookIntegrationTest {
            ok: false,
            message: format!("Invoice Ninja error ({status}): {text}"),
        });
    }
    Ok(CashbookIntegrationTest {
        ok: true,
        message: "Connected to Invoice Ninja".into(),
    })
}

#[tauri::command]
pub fn list_cashbook_entries(
    state: State<'_, AppState>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<Vec<CashbookEntry>, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;

    let sql = "SELECT e.id, e.school_id, e.direction, e.category, e.amount, e.currency,
                      e.description, e.user_id, u.name, e.course_id, c.title,
                      e.payment_method, e.reference, e.entry_date, e.created_by, cb.name, e.created_at
               FROM cashbook_entries e
               LEFT JOIN users u ON u.id = e.user_id
               LEFT JOIN courses c ON c.id = e.course_id
               LEFT JOIN users cb ON cb.id = e.created_by
               WHERE e.school_id = ?1
                 AND (?2 IS NULL OR e.entry_date >= ?2)
                 AND (?3 IS NULL OR e.entry_date <= ?3)
               ORDER BY e.entry_date DESC, e.created_at DESC";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![school_id, from_date, to_date], row_to_entry)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn create_cashbook_entry(
    state: State<'_, AppState>,
    input: CreateCashbookEntryInput,
) -> Result<CashbookEntry, String> {
    require_admin(&state)?;
    validate_entry(&input)?;
    let user = require_user(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    let settings = read_settings(&conn);
    let currency = if input.currency.trim().is_empty() {
        settings.currency
    } else {
        input.currency.trim().to_uppercase()
    };

    if let Some(ref uid) = input.user_id {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM school_members WHERE school_id = ?1 AND user_id = ?2",
                params![school_id, uid],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err("User not found in active school".into());
        }
    }
    if let Some(ref cid) = input.course_id {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM courses WHERE id = ?1 AND school_id = ?2",
                params![cid, school_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err("Course not found in active school".into());
        }
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let entry_date = if input.entry_date.trim().is_empty() {
        Utc::now().format("%Y-%m-%d").to_string()
    } else {
        input.entry_date.trim().to_string()
    };

    conn.execute(
        "INSERT INTO cashbook_entries
         (id, school_id, direction, category, amount, currency, description, user_id, course_id,
          payment_method, reference, entry_date, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            id,
            school_id,
            input.direction,
            input.category,
            input.amount,
            currency,
            input.description,
            input.user_id,
            input.course_id,
            input.payment_method,
            input.reference,
            entry_date,
            user.id,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT e.id, e.school_id, e.direction, e.category, e.amount, e.currency,
                e.description, e.user_id, u.name, e.course_id, c.title,
                e.payment_method, e.reference, e.entry_date, e.created_by, cb.name, e.created_at
         FROM cashbook_entries e
         LEFT JOIN users u ON u.id = e.user_id
         LEFT JOIN courses c ON c.id = e.course_id
         LEFT JOIN users cb ON cb.id = e.created_by
         WHERE e.id = ?1",
        params![id],
        row_to_entry,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_cashbook_entry(state: State<'_, AppState>, id: String) -> Result<(), String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    let n = conn
        .execute(
            "DELETE FROM cashbook_entries WHERE id = ?1 AND school_id = ?2",
            params![id, school_id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("Entry not found".into());
    }
    Ok(())
}

#[tauri::command]
pub fn get_cashbook_summary(
    state: State<'_, AppState>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<CashbookSummary, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;
    let settings = read_settings(&conn);

    let income: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM cashbook_entries
             WHERE school_id = ?1 AND direction = 'income'
               AND (?2 IS NULL OR entry_date >= ?2)
               AND (?3 IS NULL OR entry_date <= ?3)",
            params![school_id, from_date, to_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let expense: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM cashbook_entries
             WHERE school_id = ?1 AND direction = 'expense'
               AND (?2 IS NULL OR entry_date >= ?2)
               AND (?3 IS NULL OR entry_date <= ?3)",
            params![school_id, from_date, to_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let student_fees: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM cashbook_entries
             WHERE school_id = ?1 AND category = 'student_fee'
               AND (?2 IS NULL OR entry_date >= ?2)
               AND (?3 IS NULL OR entry_date <= ?3)",
            params![school_id, from_date, to_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let teacher_salary: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM cashbook_entries
             WHERE school_id = ?1 AND category = 'teacher_salary'
               AND (?2 IS NULL OR entry_date >= ?2)
               AND (?3 IS NULL OR entry_date <= ?3)",
            params![school_id, from_date, to_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let entry_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cashbook_entries
             WHERE school_id = ?1
               AND (?2 IS NULL OR entry_date >= ?2)
               AND (?3 IS NULL OR entry_date <= ?3)",
            params![school_id, from_date, to_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(CashbookSummary {
        currency: settings.currency,
        total_income: income,
        total_expense: expense,
        balance: income - expense,
        student_fees,
        teacher_salary,
        entry_count,
    })
}

#[tauri::command]
pub fn export_cashbook_csv(
    state: State<'_, AppState>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<String, String> {
    require_admin(&state)?;
    let entries = list_cashbook_entries(state, from_date, to_date)?;
    let mut lines = vec![
        "date,direction,category,amount,currency,payment_method,user,course,description,reference"
            .to_string(),
    ];
    for e in entries {
        lines.push(format!(
            "{},{},{},{:.2},{},{},{},{},{},{}",
            e.entry_date,
            e.direction,
            e.category,
            e.amount,
            e.currency,
            e.payment_method,
            csv_escape(e.user_name.as_deref().unwrap_or("")),
            csv_escape(e.course_title.as_deref().unwrap_or("")),
            csv_escape(e.description.as_deref().unwrap_or("")),
            csv_escape(e.reference.as_deref().unwrap_or("")),
        ));
    }
    Ok(lines.join("\n"))
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
