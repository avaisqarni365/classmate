use crate::models::{CreateForumPostInput, CreateForumTopicInput, ForumPost, ForumTopic};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_forum_topics(
    state: State<'_, AppState>,
    course_id: String,
) -> Result<Vec<ForumTopic>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.course_id, t.title, t.author_name, COUNT(p.id) AS post_count, t.created_at
             FROM forum_topics t
             LEFT JOIN forum_posts p ON p.topic_id = t.id
             WHERE t.course_id = ?1
             GROUP BY t.id
             ORDER BY t.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![course_id], |row| {
            Ok(ForumTopic {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                author_name: row.get(3)?,
                post_count: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn list_forum_posts(
    state: State<'_, AppState>,
    topic_id: String,
) -> Result<Vec<ForumPost>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, topic_id, author_name, body, created_at
             FROM forum_posts WHERE topic_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![topic_id], |row| {
            Ok(ForumPost {
                id: row.get(0)?,
                topic_id: row.get(1)?,
                author_name: row.get(2)?,
                body: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn create_forum_topic(
    state: State<'_, AppState>,
    input: CreateForumTopicInput,
) -> Result<ForumTopic, String> {
    let now = Utc::now().to_rfc3339();
    let topic_id = Uuid::new_v4().to_string();
    let post_id = Uuid::new_v4().to_string();

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO forum_topics (id, course_id, title, author_name, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            topic_id,
            input.course_id,
            input.title.trim(),
            input.author_name.trim(),
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO forum_posts (id, topic_id, author_name, body, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            post_id,
            topic_id,
            input.author_name.trim(),
            input.body.trim(),
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(ForumTopic {
        id: topic_id,
        course_id: input.course_id,
        title: input.title.trim().to_string(),
        author_name: input.author_name.trim().to_string(),
        post_count: 1,
        created_at: now,
    })
}

#[tauri::command]
pub fn create_forum_post(
    state: State<'_, AppState>,
    input: CreateForumPostInput,
) -> Result<ForumPost, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO forum_posts (id, topic_id, author_name, body, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            id,
            input.topic_id,
            input.author_name.trim(),
            input.body.trim(),
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(ForumPost {
        id,
        topic_id: input.topic_id,
        author_name: input.author_name.trim().to_string(),
        body: input.body.trim().to_string(),
        created_at: now,
    })
}
