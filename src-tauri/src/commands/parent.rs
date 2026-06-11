use crate::commands::rubrics::rubric_scores_display;
use crate::models::{CertificateInfo, IssueCertificateInput, LinkParentInput, ParentDigest, ParentGradeEntry, ParentStudentSummary};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn link_parent_student(
    state: State<'_, AppState>,
    input: LinkParentInput,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO parent_links (id, parent_id, student_id, created_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(parent_id, student_id) DO NOTHING",
        params![Uuid::new_v4().to_string(), input.parent_id, input.student_id, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_parent_dashboard(
    state: State<'_, AppState>,
    parent_id: String,
) -> Result<Vec<ParentStudentSummary>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    build_parent_dashboard(&conn, &parent_id)
}

fn build_parent_dashboard(
    conn: &rusqlite::Connection,
    parent_id: &str,
) -> Result<Vec<ParentStudentSummary>, String> {
    let mut student_stmt = conn
        .prepare(
            "SELECT u.id, u.name FROM parent_links pl
             JOIN users u ON u.id = pl.student_id
             WHERE pl.parent_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let students = student_stmt
        .query_map(params![parent_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for (student_id, student_name) in students {
        let mut course_stmt = conn
            .prepare(
                "SELECT c.id, c.title FROM enrollments e
                 JOIN courses c ON c.id = e.course_id
                 WHERE e.student_id = ?1 AND e.status = 'active'",
            )
            .map_err(|e| e.to_string())?;

        let courses = course_stmt
            .query_map(params![student_id.clone()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut course_summaries = Vec::new();
        for (course_id, course_title) in courses {
            let (assignment_count, avg): (i64, Option<f64>) = conn
                .query_row(
                    "SELECT COUNT(a.id),
                            AVG(CASE WHEN g.points IS NOT NULL AND a.max_points > 0
                                THEN (g.points / a.max_points) * 100 END)
                     FROM assignments a
                     LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = ?2
                     WHERE a.course_id = ?1",
                    params![course_id, student_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap_or((0, None));

            course_summaries.push(crate::models::ParentCourseSummary {
                course_id,
                course_title,
                average_percent: avg,
                assignment_count,
            });
        }

        out.push(ParentStudentSummary {
            student_id,
            student_name,
            courses: course_summaries,
        });
    }

    Ok(out)
}

#[tauri::command]
pub fn list_parent_grades(
    state: State<'_, AppState>,
    parent_id: String,
) -> Result<Vec<ParentGradeEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT u.id, u.name, c.title, a.id, a.title, g.points, a.max_points, g.feedback, g.graded_at, g.rubric_scores_json
             FROM parent_links pl
             JOIN users u ON u.id = pl.student_id
             JOIN enrollments e ON e.student_id = u.id AND e.status = 'active'
             JOIN courses c ON c.id = e.course_id
             JOIN assignments a ON a.course_id = c.id
             LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = u.id
             WHERE pl.parent_id = ?1 AND g.points IS NOT NULL
             ORDER BY g.graded_at DESC, a.title ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![parent_id], |row| {
            let assignment_id: String = row.get(3)?;
            let rubric_scores_json: Option<String> = row.get(9)?;
            let rubric_scores = rubric_scores_display(
                &conn,
                &assignment_id,
                rubric_scores_json.as_deref(),
            )
            .ok()
            .flatten();
            Ok(ParentGradeEntry {
                student_id: row.get(0)?,
                student_name: row.get(1)?,
                course_title: row.get(2)?,
                assignment_id,
                assignment_title: row.get(4)?,
                points: row.get(5)?,
                max_points: row.get(6)?,
                feedback: row.get(7)?,
                graded_at: row.get(8)?,
                rubric_scores,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn generate_parent_digest(
    state: State<'_, AppState>,
    parent_id: String,
) -> Result<ParentDigest, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    build_parent_digest(&conn, &parent_id)
}

pub fn build_parent_digest(
    conn: &rusqlite::Connection,
    parent_id: &str,
) -> Result<ParentDigest, String> {
    let parent_name: String = conn
        .query_row(
            "SELECT name FROM users WHERE id = ?1 AND role = 'parent'",
            params![parent_id],
            |row| row.get(0),
        )
        .map_err(|_| "Parent not found".to_string())?;

    let dashboard = build_parent_dashboard(&conn, &parent_id)?;
    let generated_at = Utc::now().to_rfc3339();
    let mut sections = String::new();

    for student in &dashboard {
        sections.push_str(&format!("<h2>{}</h2>", html_escape(&student.student_name)));

        for course in &student.courses {
            let avg = course
                .average_percent
                .map(|v| format!("{v:.1}%"))
                .unwrap_or_else(|| "No grades yet".into());
            sections.push_str(&format!(
                "<h3>{}</h3><p>Average: {avg} · {} assignments</p>",
                html_escape(&course.course_title),
                course.assignment_count
            ));

            let mut grade_stmt = conn
                .prepare(
                    "SELECT a.id, a.title, g.points, a.max_points, g.graded_at, g.rubric_scores_json FROM assignments a LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = ?2 WHERE a.course_id = ?1 ORDER BY g.graded_at DESC LIMIT 5",
                )
                .map_err(|e| e.to_string())?;
            sections.push_str("<ul>");
            for row in grade_stmt
                .query_map(params![course.course_id, student.student_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<f64>>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .flatten()
            {
                let (assignment_id, title, points, max_points, graded_at, rubric_json) = row;
                let mut line = if let Some(pts) = points {
                    format!(
                        "{title}: {pts:.1}/{max_points:.1} ({})",
                        graded_at.unwrap_or_default()
                    )
                } else {
                    format!("{title}: not graded")
                };
                if let Some(rubric_line) =
                    format_rubric_line(&conn, &assignment_id, rubric_json.as_deref())
                {
                    line.push_str(&format!(" — {rubric_line}"));
                }
                sections.push_str(&format!("<li>{}</li>", html_escape(&line)));
            }
            sections.push_str("</ul>");

            let upcoming: Vec<(String, Option<String>)> = conn
                .prepare(
                    "SELECT a.title, a.due_at FROM assignments a LEFT JOIN grades g ON g.assignment_id = a.id AND g.student_id = ?2 WHERE a.course_id = ?1 AND g.points IS NULL ORDER BY a.due_at ASC LIMIT 3",
                )
                .map_err(|e| e.to_string())?
                .query_map(params![course.course_id, student.student_id], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })
                .map_err(|e| e.to_string())?
                .flatten()
                .collect();
            if !upcoming.is_empty() {
                sections.push_str("<p><strong>Upcoming work</strong></p><ul>");
                for (title, due) in upcoming {
                    let due_text = due.unwrap_or_else(|| "No due date".into());
                    sections.push_str(&format!(
                        "<li>{} — {}</li>",
                        html_escape(&title),
                        html_escape(&due_text)
                    ));
                }
                sections.push_str("</ul>");
            }
        }
    }

    if dashboard.is_empty() {
        sections.push_str("<p>No linked students yet.</p>");
    }

    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Weekly digest</title>
<style>body{{font-family:system-ui,sans-serif;max-width:760px;margin:2rem auto;padding:0 1rem;color:#0f172a}}
h1{{color:#1d4ed8}}h2{{margin-top:1.5rem;border-bottom:1px solid #e2e8f0;padding-bottom:.35rem}}
h3{{margin-bottom:.25rem}}ul{{padding-left:1.2rem}}p{{color:#475569}}</style></head>
<body><h1>ClassMate weekly digest</h1><p>For {parent_name} · Generated {generated_at}</p>{sections}</body></html>"#
    );

    Ok(ParentDigest {
        parent_name,
        generated_at,
        html,
    })
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn format_rubric_line(
    conn: &rusqlite::Connection,
    assignment_id: &str,
    rubric_scores_json: Option<&str>,
) -> Option<String> {
    let scores = rubric_scores_display(conn, assignment_id, rubric_scores_json).ok()??;
    Some(
        scores
            .iter()
            .map(|s| format!("{} {}/{}", s.name, s.points, s.max_points))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

#[tauri::command]
pub fn issue_certificate(
    state: State<'_, AppState>,
    input: IssueCertificateInput,
) -> Result<CertificateInfo, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let (course_title, student_name): (String, String) = conn
        .query_row(
            "SELECT c.title, u.name FROM courses c, users u
             WHERE c.id = ?1 AND u.id = ?2",
            params![input.course_id, input.student_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Course or student not found".to_string())?;

    conn.execute(
        "INSERT INTO certificates (id, course_id, student_id, issued_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(course_id, student_id) DO UPDATE SET issued_at = excluded.issued_at",
        params![id, input.course_id, input.student_id, now],
    )
    .map_err(|e| e.to_string())?;

    let html = certificate_html(&student_name, &course_title, &now);

    Ok(CertificateInfo {
        id,
        course_id: input.course_id,
        course_title,
        student_id: input.student_id,
        student_name,
        issued_at: now,
        html,
    })
}

#[tauri::command]
pub fn list_certificates(
    state: State<'_, AppState>,
    course_id: Option<String>,
) -> Result<Vec<CertificateInfo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sql = if course_id.is_some() {
        "SELECT cert.id, cert.course_id, c.title, cert.student_id, u.name, cert.issued_at
         FROM certificates cert
         JOIN courses c ON c.id = cert.course_id
         JOIN users u ON u.id = cert.student_id
         WHERE cert.course_id = ?1 ORDER BY cert.issued_at DESC"
    } else {
        "SELECT cert.id, cert.course_id, c.title, cert.student_id, u.name, cert.issued_at
         FROM certificates cert
         JOIN courses c ON c.id = cert.course_id
         JOIN users u ON u.id = cert.student_id
         ORDER BY cert.issued_at DESC"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let map = |row: &rusqlite::Row<'_>| {
        let course_title: String = row.get(2)?;
        let student_name: String = row.get(4)?;
        let issued_at: String = row.get(5)?;
        Ok(CertificateInfo {
            id: row.get(0)?,
            course_id: row.get(1)?,
            course_title: course_title.clone(),
            student_id: row.get(3)?,
            student_name: student_name.clone(),
            issued_at: issued_at.clone(),
            html: certificate_html(&student_name, &course_title, &issued_at),
        })
    };

    let rows = if let Some(ref cid) = course_id {
        stmt.query_map(params![cid], map)
    } else {
        stmt.query_map([], map)
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(rows)
}

fn certificate_html(student: &str, course: &str, date: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Certificate</title>
<style>body{{font-family:Georgia,serif;text-align:center;padding:3rem;background:#f8fafc}}
.card{{background:#fff;border:8px double #2563eb;padding:3rem;max-width:720px;margin:0 auto}}
h1{{color:#1d4ed8;margin:0}}p{{color:#334155;font-size:1.1rem}}</style></head>
<body><div class="card"><h1>Certificate of Completion</h1>
<p>This certifies that</p><h2>{student}</h2>
<p>has successfully completed</p><h3>{course}</h3>
<p>Issued on {date}</p><p>ClassMate Local Classroom Platform</p></div></body></html>"#
    )
}
