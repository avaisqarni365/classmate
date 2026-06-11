use crate::models::{
    CreateSessionPollInput, PollResults, SessionPoll, VotePollInput,
};
use crate::AppState;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;
use uuid::Uuid;

fn map_poll_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionPoll> {
    let options_json: String = row.get(3)?;
    let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
    Ok(SessionPoll {
        id: row.get(0)?,
        session_id: row.get(1)?,
        question: row.get(2)?,
        options,
        status: row.get(4)?,
        created_at: row.get(5)?,
    })
}

pub fn poll_results(conn: &Connection, poll: SessionPoll) -> Result<PollResults, String> {
    let mut vote_counts = vec![0i64; poll.options.len()];
    let mut stmt = conn
        .prepare("SELECT option_index, COUNT(*) FROM session_poll_votes WHERE poll_id = ?1 GROUP BY option_index")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![poll.id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut total_votes = 0i64;
    for (idx, count) in rows {
        if idx >= 0 && (idx as usize) < vote_counts.len() {
            vote_counts[idx as usize] = count;
            total_votes += count;
        }
    }

    Ok(PollResults {
        poll,
        vote_counts,
        total_votes,
    })
}

fn fetch_poll(conn: &Connection, poll_id: &str) -> Result<SessionPoll, String> {
    conn.query_row(
        "SELECT id, session_id, question, options_json, status, created_at FROM session_polls WHERE id = ?1",
        params![poll_id],
        map_poll_row,
    )
    .map_err(|_| "Poll not found".to_string())
}

#[tauri::command]
pub fn create_session_poll(
    state: State<'_, AppState>,
    input: CreateSessionPollInput,
) -> Result<PollResults, String> {
    if input.question.trim().is_empty() {
        return Err("Question is required".into());
    }
    if input.options.len() < 2 {
        return Err("At least two options required".into());
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE session_polls SET status = 'closed' WHERE session_id = ?1 AND status = 'open'",
        params![input.session_id],
    )
    .map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let options_json =
        serde_json::to_string(&input.options).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO session_polls (id, session_id, question, options_json, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'open', ?5)",
        params![id, input.session_id, input.question.trim(), options_json, now],
    )
    .map_err(|e| e.to_string())?;

    let poll = fetch_poll(&conn, &id)?;
    poll_results(&conn, poll)
}

#[tauri::command]
pub fn close_session_poll(state: State<'_, AppState>, poll_id: String) -> Result<PollResults, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE session_polls SET status = 'closed' WHERE id = ?1",
        params![poll_id],
    )
    .map_err(|e| e.to_string())?;
    let poll = fetch_poll(&conn, &poll_id)?;
    poll_results(&conn, poll)
}

#[tauri::command]
pub fn get_active_session_poll(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<PollResults>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let poll = conn
        .query_row(
            "SELECT id, session_id, question, options_json, status, created_at
             FROM session_polls WHERE session_id = ?1 AND status = 'open'
             ORDER BY created_at DESC LIMIT 1",
            params![session_id],
            map_poll_row,
        )
        .optional()
        .map_err(|e| e.to_string())?;

    match poll {
        Some(p) => Ok(Some(poll_results(&conn, p)?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_session_poll_results(
    state: State<'_, AppState>,
    poll_id: String,
) -> Result<PollResults, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let poll = fetch_poll(&conn, &poll_id)?;
    poll_results(&conn, poll)
}

pub fn cast_poll_vote(conn: &Connection, input: VotePollInput) -> Result<PollResults, String> {
    if input.student_name.trim().is_empty() {
        return Err("Student name is required".into());
    }

    let poll = fetch_poll(&conn, &input.poll_id)?;
    if poll.status != "open" {
        return Err("Poll is closed".into());
    }
    if input.option_index < 0 || input.option_index as usize >= poll.options.len() {
        return Err("Invalid option".into());
    }

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO session_poll_votes (id, poll_id, student_name, option_index, voted_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(poll_id, student_name) DO UPDATE SET option_index = excluded.option_index, voted_at = excluded.voted_at",
        params![
            Uuid::new_v4().to_string(),
            input.poll_id,
            input.student_name.trim(),
            input.option_index,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    poll_results(conn, poll)
}

#[tauri::command]
pub fn vote_session_poll(
    state: State<'_, AppState>,
    input: VotePollInput,
) -> Result<PollResults, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    cast_poll_vote(&conn, input)
}
