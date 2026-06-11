use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;

const MIGRATIONS: &str = include_str!("schema.sql");
const MIGRATIONS_V3: &str = include_str!("schema_v3.sql");
const MIGRATIONS_V4: &str = include_str!("schema_v4.sql");
const MIGRATIONS_V5: &str = include_str!("schema_v5.sql");
const MIGRATIONS_V6: &str = include_str!("schema_v6.sql");
const MIGRATIONS_V7: &str = include_str!("schema_v7.sql");
const MIGRATIONS_V8: &str = include_str!("schema_v8.sql");
const MIGRATIONS_V9: &str = include_str!("schema_v9.sql");
const MIGRATIONS_V10: &str = include_str!("schema_v10.sql");
const MIGRATIONS_V11: &str = include_str!("schema_v11.sql");
const MIGRATIONS_V12: &str = include_str!("schema_v12.sql");
const MIGRATIONS_V13: &str = include_str!("schema_v13.sql");
const MIGRATIONS_V14: &str = include_str!("schema_v14.sql");
const MIGRATIONS_V15: &str = include_str!("schema_v15.sql");
const MIGRATIONS_V16: &str = include_str!("schema_v16.sql");
const MIGRATIONS_V17: &str = include_str!("schema_v17.sql");
const MIGRATIONS_V18: &str = include_str!("schema_v18.sql");
const MIGRATIONS_V19: &str = include_str!("schema_v19.sql");
const MIGRATIONS_V20: &str = include_str!("schema_v20.sql");

pub fn init(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    conn.execute_batch(MIGRATIONS)?;
    conn.execute_batch(MIGRATIONS_V3)?;
    conn.execute_batch(MIGRATIONS_V4)?;
    conn.execute_batch(MIGRATIONS_V5)?;
    migrate_v6(&conn)?;
    migrate_v7(&conn)?;
    migrate_v8(&conn)?;
    migrate_v9(&conn)?;
    migrate_v10(&conn)?;
    migrate_v11(&conn)?;
    migrate_v12(&conn)?;
    migrate_v13(&conn)?;
    migrate_v14(&conn)?;
    migrate_v15(&conn)?;
    migrate_v16(&conn)?;
    migrate_v17(&conn)?;
    migrate_v18(&conn)?;
    migrate_v19(&conn)?;
    migrate_v20(&conn)?;
    seed_if_empty(&conn)?;
    Ok(conn)
}

fn migrate_v6(conn: &Connection) -> rusqlite::Result<()> {
    let has_col: bool = conn
        .prepare("PRAGMA table_info(quiz_questions)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|name| name == "correct_text");
    if !has_col {
        conn.execute_batch(MIGRATIONS_V6)?;
    }
    Ok(())
}

fn migrate_v7(conn: &Connection) -> rusqlite::Result<()> {
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(quiz_attempts)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !cols.iter().any(|name| name == "review_status") {
        conn.execute_batch(MIGRATIONS_V7)?;
    }
    Ok(())
}

fn migrate_v8(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='assignment_rubrics'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V8)?;
    }
    Ok(())
}

fn migrate_v9(conn: &Connection) -> rusqlite::Result<()> {
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(users)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !cols.iter().any(|name| name == "phone") {
        conn.execute_batch(MIGRATIONS_V9)?;
    }
    Ok(())
}

fn migrate_v10(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='whatsapp_outbound_messages'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V10)?;
    }
    Ok(())
}

fn migrate_v11(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='email_log'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V11)?;
    }
    Ok(())
}

fn migrate_v12(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='whatsapp_inbound_messages'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V12)?;
    }
    Ok(())
}

fn migrate_v13(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='whatsapp_scheduled_broadcasts'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V13)?;
    }
    Ok(())
}

fn migrate_v14(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='whatsapp_consent_log'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V14)?;
    }
    Ok(())
}

fn migrate_v15(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='school_members'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V15)?;
        backfill_tenancy(conn)?;
    }
    Ok(())
}

fn backfill_tenancy(conn: &Connection) -> rusqlite::Result<()> {
    let school_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM schools", [], |row| row.get(0))?;
    let default_school_id = if school_count == 0 {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO schools (id, name, code, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, "Main Campus", "MAIN", now],
        )?;
        id
    } else {
        conn.query_row(
            "SELECT id FROM schools ORDER BY created_at ASC LIMIT 1",
            [],
            |row| row.get(0),
        )?
    };

    conn.execute(
        "UPDATE courses SET school_id = ?1 WHERE school_id IS NULL",
        params![default_school_id],
    )?;

    let user_ids: Vec<String> = conn
        .prepare("SELECT id FROM users")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let now = Utc::now().to_rfc3339();
    for user_id in user_ids {
        conn.execute(
            "INSERT OR IGNORE INTO school_members (id, school_id, user_id, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                Uuid::new_v4().to_string(),
                default_school_id,
                user_id,
                now
            ],
        )?;
    }

    Ok(())
}

fn migrate_v16(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='push_devices'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V16)?;
    }
    Ok(())
}

fn migrate_v17(conn: &Connection) -> rusqlite::Result<()> {
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(whatsapp_group_links)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !cols.iter().any(|name| name == "external_group_id") {
        conn.execute_batch(MIGRATIONS_V17)?;
    }
    Ok(())
}

fn migrate_v18(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='cashbook_entries'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V18)?;
    }
    Ok(())
}

fn migrate_v19(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='whatsapp_group_participant_events'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V19)?;
    }
    Ok(())
}

fn migrate_v20(conn: &Connection) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='whatsapp_group_settings_events'",
        [],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(MIGRATIONS_V20)?;
    }
    Ok(())
}

fn seed_if_empty(conn: &Connection) -> rusqlite::Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();
    let admin_id = Uuid::new_v4().to_string();
    let teacher_id = Uuid::new_v4().to_string();
    let student_id = Uuid::new_v4().to_string();
    let parent_id = Uuid::new_v4().to_string();
    let course_id = Uuid::new_v4().to_string();
    let assignment_id = Uuid::new_v4().to_string();
    let main_school_id = Uuid::new_v4().to_string();
    let east_school_id = Uuid::new_v4().to_string();

    let admin_hash = hash("admin123", DEFAULT_COST).unwrap_or_else(|_| "invalid".into());
    let teacher_hash = hash("teacher123", DEFAULT_COST).unwrap_or_else(|_| "invalid".into());
    let student_hash = hash("student123", DEFAULT_COST).unwrap_or_else(|_| "invalid".into());
    let parent_hash = hash("parent123", DEFAULT_COST).unwrap_or_else(|_| "invalid".into());

    conn.execute(
        "INSERT INTO schools (id, name, code, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![main_school_id, "Main Campus", "MAIN", now],
    )?;
    conn.execute(
        "INSERT INTO schools (id, name, code, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![east_school_id, "East Campus", "EAST", now],
    )?;

    conn.execute(
        "INSERT INTO users (id, email, name, role, password_hash, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            admin_id,
            "admin@classmate.local",
            "System Admin",
            "admin",
            admin_hash,
            now,
            now
        ],
    )?;

    conn.execute(
        "INSERT INTO users (id, email, name, role, password_hash, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            teacher_id,
            "teacher@classmate.local",
            "Demo Teacher",
            "teacher",
            teacher_hash,
            now,
            now
        ],
    )?;

    conn.execute(
        "INSERT INTO users (id, email, name, role, password_hash, phone, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            student_id,
            "student@classmate.local",
            "Demo Student",
            "student",
            student_hash,
            "+15551234567",
            now,
            now
        ],
    )?;

    conn.execute(
        "INSERT INTO whatsapp_consent (user_id, opted_in, opted_in_at, source)
         VALUES (?1, 1, ?2, 'seed')",
        params![student_id, now],
    )?;

    conn.execute(
        "INSERT INTO users (id, email, name, role, password_hash, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            parent_id,
            "parent@classmate.local",
            "Demo Parent",
            "parent",
            parent_hash,
            now,
            now
        ],
    )?;

    conn.execute(
        "INSERT INTO parent_links (id, parent_id, student_id, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![Uuid::new_v4().to_string(), parent_id, student_id, now],
    )?;

    for user_id in [&admin_id, &teacher_id, &student_id, &parent_id] {
        conn.execute(
            "INSERT INTO school_members (id, school_id, user_id, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![Uuid::new_v4().to_string(), main_school_id, user_id, now],
        )?;
    }
    conn.execute(
        "INSERT INTO school_members (id, school_id, user_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![Uuid::new_v4().to_string(), east_school_id, admin_id, now],
    )?;

    conn.execute(
        "INSERT INTO courses (id, title, code, description, teacher_id, term, school_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            course_id,
            "Introduction to Science",
            "SCI-101",
            "Sample course for local classroom demos.",
            teacher_id,
            "2026 Spring",
            main_school_id,
            now,
            now
        ],
    )?;

    conn.execute(
        "INSERT INTO enrollments (id, course_id, student_id, status, enrolled_at)
         VALUES (?1, ?2, ?3, 'active', ?4)",
        params![Uuid::new_v4().to_string(), course_id, student_id, now],
    )?;

    conn.execute(
        "INSERT INTO assignments (id, course_id, title, description, due_at, max_points, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            assignment_id,
            course_id,
            "Lab Report 1",
            "Submit your first lab observations.",
            now,
            100.0,
            now
        ],
    )?;

    conn.execute(
        "INSERT INTO assignment_rubrics (assignment_id, criteria_json, updated_at) VALUES (?1, ?2, ?3)",
        params![
            assignment_id,
            r#"[{"id":"crit-1","name":"Observations","description":"Accurate lab notes","max_points":50},{"id":"crit-2","name":"Analysis","description":"Clear conclusions","max_points":50}]"#,
            now
        ],
    )?;

    conn.execute(
        "INSERT INTO course_materials (id, course_id, title, kind, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            Uuid::new_v4().to_string(),
            course_id,
            "Safety guidelines",
            "note",
            "Always wear goggles in the lab. Report spills immediately.",
            now
        ],
    )?;

    let quiz_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO quizzes (id, course_id, title, description, time_limit_minutes, max_points, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'published', ?7)",
        params![
            quiz_id,
            course_id,
            "Science Basics Check",
            "Quick check on lab safety and the scientific method.",
            15,
            2.0,
            now
        ],
    )?;
    conn.execute(
        "INSERT INTO quiz_questions (id, quiz_id, prompt, kind, options_json, correct_index, points, sort_order)
         VALUES (?1, ?2, ?3, 'mcq', ?4, ?5, ?6, ?7)",
        params![
            Uuid::new_v4().to_string(),
            quiz_id,
            "What should you wear in the lab?",
            r#"["Regular clothes","Safety goggles","Sunglasses","Open sandals"]"#,
            1,
            1.0,
            0
        ],
    )?;
    conn.execute(
        "INSERT INTO quiz_questions (id, quiz_id, prompt, kind, options_json, correct_index, points, sort_order)
         VALUES (?1, ?2, ?3, 'mcq', ?4, ?5, ?6, ?7)",
        params![
            Uuid::new_v4().to_string(),
            quiz_id,
            "The first step of the scientific method is:",
            r#"["Conclusion","Observation","Publishing","Ignore data"]"#,
            1,
            1.0,
            1
        ],
    )?;
    conn.execute(
        "INSERT INTO quiz_questions (id, quiz_id, prompt, kind, options_json, correct_index, correct_text, points, sort_order)
         VALUES (?1, ?2, ?3, 'short_answer', '[]', 0, ?4, ?5, ?6)",
        params![
            Uuid::new_v4().to_string(),
            quiz_id,
            "Name the first step of the scientific method.",
            "Observation",
            1.0,
            2
        ],
    )?;

    conn.execute(
        "INSERT INTO schedule_slots (id, course_id, day_of_week, start_time, end_time, room, title, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            Uuid::new_v4().to_string(),
            course_id,
            1,
            "09:00",
            "10:30",
            "Lab 101",
            "Morning lab",
            now
        ],
    )?;
    conn.execute(
        "INSERT INTO schedule_slots (id, course_id, day_of_week, start_time, end_time, room, title, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            Uuid::new_v4().to_string(),
            course_id,
            3,
            "13:00",
            "14:30",
            "Lab 101",
            "Afternoon review",
            now
        ],
    )?;

    Ok(())
}
