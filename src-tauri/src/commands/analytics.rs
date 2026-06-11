use crate::commands::tenancy::active_school_id_or_resolve;
use crate::models::{AnalyticsReport, AtRiskStudent, CourseAnalytics, SessionSummary};
use crate::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_analytics(state: State<'_, AppState>) -> Result<AnalyticsReport, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let school_id = active_school_id_or_resolve(&state, &conn)?;

    let mut risk_stmt = conn
        .prepare(
            "SELECT u.id, u.name, c.id, c.title,
                    AVG(CASE WHEN g.points IS NOT NULL AND a.max_points > 0
                        THEN (g.points / a.max_points) * 100 END) AS avg_pct
             FROM users u
             JOIN enrollments e ON e.student_id = u.id AND e.status = 'active'
             JOIN courses c ON c.id = e.course_id
             JOIN assignments a ON a.course_id = c.id
             LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = u.id
             WHERE u.role = 'student' AND c.school_id = ?1
             GROUP BY u.id, c.id
             HAVING avg_pct IS NOT NULL AND avg_pct < 60
             ORDER BY avg_pct ASC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let at_risk_students = risk_stmt
        .query_map(params![school_id], |row| {
            Ok(AtRiskStudent {
                student_id: row.get(0)?,
                student_name: row.get(1)?,
                course_id: row.get(2)?,
                course_title: row.get(3)?,
                average_percent: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut course_stmt = conn
        .prepare(
            "SELECT c.id, c.title, COUNT(DISTINCT e.student_id),
                    AVG(CASE WHEN g.points IS NOT NULL AND a.max_points > 0
                        THEN (g.points / a.max_points) * 100 END)
             FROM courses c
             LEFT JOIN enrollments e ON e.course_id = c.id AND e.status = 'active'
             LEFT JOIN assignments a ON a.course_id = c.id
             LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = e.student_id
             WHERE c.school_id = ?1
             GROUP BY c.id ORDER BY c.title",
        )
        .map_err(|e| e.to_string())?;

    let course_rows: Vec<(String, String, i64, Option<f64>)> = course_stmt
        .query_map(params![school_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut course_summaries = Vec::new();
    for (course_id, course_title, student_count, average_percent) in course_rows {
        let attendance_rate: Option<f64> = conn
            .query_row(
                "SELECT CASE WHEN COUNT(DISTINCT cs.id) > 0
                    THEN COUNT(DISTINCT att.id) * 100.0 / COUNT(DISTINCT cs.id) END
                 FROM class_sessions cs
                 LEFT JOIN attendance att ON att.session_id = cs.id
                 WHERE cs.course_id = ?1",
                params![course_id],
                |row| row.get(0),
            )
            .ok()
            .flatten();
        course_summaries.push(CourseAnalytics {
            course_id,
            course_title,
            student_count,
            average_percent,
            attendance_rate,
        });
    }

    let mut session_stmt = conn
        .prepare(
            "SELECT cs.id, c.title, cs.title, COUNT(att.id), cs.started_at
             FROM class_sessions cs
             JOIN courses c ON c.id = cs.course_id
             LEFT JOIN attendance att ON att.session_id = cs.id
             WHERE c.school_id = ?1
             GROUP BY cs.id ORDER BY cs.started_at DESC LIMIT 10",
        )
        .map_err(|e| e.to_string())?;

    let recent_sessions = session_stmt
        .query_map(params![school_id], |row| {
            Ok(SessionSummary {
                session_id: row.get(0)?,
                course_title: row.get(1)?,
                title: row.get(2)?,
                attendance_count: row.get(3)?,
                started_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let total_quiz_attempts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM quiz_attempts qa
             JOIN quizzes q ON q.id = qa.quiz_id
             JOIN courses c ON c.id = q.course_id
             WHERE c.school_id = ?1",
            params![school_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(AnalyticsReport {
        at_risk_students,
        course_summaries,
        recent_sessions,
        total_quiz_attempts,
    })
}

#[tauri::command]
pub fn export_attendance_csv(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (course_title, session_title): (String, String) = conn
        .query_row(
            "SELECT c.title, cs.title FROM class_sessions cs
             JOIN courses c ON c.id = cs.course_id WHERE cs.id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Session not found".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT student_name, joined_at FROM attendance
             WHERE session_id = ?1 ORDER BY joined_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let mut lines = vec!["Course,Session,Student,Joined At".to_string()];
    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (name, joined) in rows {
        lines.push(format!(
            "{},{},{},{}",
            escape_csv(&course_title),
            escape_csv(&session_title),
            escape_csv(&name),
            joined
        ));
    }

    Ok(lines.join("\n"))
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
