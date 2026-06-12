use crate::commands::rubrics::rubric_for_assignment;
use crate::models::{
    Assignment, CreateAssignmentInput, GradeCell, Gradebook, GradebookStudent, RubricScoreInput,
    SaveGradeInput,
};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn create_assignment(
    state: State<'_, AppState>,
    input: CreateAssignmentInput,
) -> Result<Assignment, String> {
    if input.title.trim().is_empty() {
        return Err("Title is required".into());
    }

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let max_points = if input.max_points > 0.0 {
        input.max_points
    } else {
        100.0
    };

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO assignments (id, course_id, title, description, due_at, max_points, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            input.course_id,
            input.title.trim(),
            input.description,
            input.due_at,
            max_points,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Assignment {
        id,
        course_id: input.course_id,
        title: input.title.trim().to_string(),
        description: input.description,
        due_at: input.due_at,
        max_points,
        created_at: now,
    })
}

#[tauri::command]
pub fn get_gradebook(state: State<'_, AppState>, course_id: String) -> Result<Gradebook, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    build_gradebook(&conn, course_id)
}

pub fn build_gradebook(conn: &rusqlite::Connection, course_id: String) -> Result<Gradebook, String> {
    let course_title: String = conn
        .query_row(
            "SELECT title FROM courses WHERE id = ?1",
            params![course_id],
            |row| row.get(0),
        )
        .map_err(|_| "Course not found".to_string())?;

    let mut assignment_stmt = conn
        .prepare(
            "SELECT id, course_id, title, description, due_at, max_points, created_at
             FROM assignments WHERE course_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let assignments = assignment_stmt
        .query_map(params![course_id], |row| {
            Ok(Assignment {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                due_at: row.get(4)?,
                max_points: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut student_stmt = conn
        .prepare(
            "SELECT u.id, u.name
             FROM enrollments e
             JOIN users u ON u.id = e.student_id
             WHERE e.course_id = ?1 AND e.status = 'active'
             ORDER BY u.name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let students_raw = student_stmt
        .query_map(params![course_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut grade_stmt = conn
        .prepare(
            "SELECT assignment_id, points, feedback, graded_at
             FROM grades WHERE student_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let mut students = Vec::new();
    for (student_id, student_name) in students_raw {
        let grade_rows = grade_stmt
            .query_map(params![student_id.clone()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    GradeCell {
                        assignment_id: row.get(0)?,
                        points: row.get(1)?,
                        feedback: row.get(2)?,
                        graded_at: row.get(3)?,
                    },
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let grades: HashMap<String, GradeCell> = grade_rows.into_iter().collect();
        students.push(GradebookStudent {
            student_id,
            student_name,
            grades,
        });
    }

    Ok(Gradebook {
        course_id,
        course_title,
        assignments,
        students,
    })
}

#[tauri::command]
pub fn save_grade(state: State<'_, AppState>, input: SaveGradeInput) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    save_grade_work(&conn, &input)
}

pub fn save_grade_work(conn: &rusqlite::Connection, input: &SaveGradeInput) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM grades WHERE assignment_id = ?1 AND student_id = ?2",
            params![input.assignment_id, input.student_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        conn.execute(
            "UPDATE grades SET points = ?1, feedback = ?2, graded_at = ?3 WHERE id = ?4",
            params![input.points, input.feedback, now, id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO grades (id, assignment_id, student_id, points, feedback, graded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                Uuid::new_v4().to_string(),
                input.assignment_id,
                input.student_id,
                input.points,
                input.feedback,
                now
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn export_gradebook_csv(state: State<'_, AppState>, course_id: String) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let book = build_gradebook(&conn, course_id)?;

    struct CsvColumn {
        header: String,
        assignment_id: Option<String>,
        criterion_id: Option<String>,
    }

    let mut columns = vec![CsvColumn {
        header: "Student".into(),
        assignment_id: None,
        criterion_id: None,
    }];

    for assignment in &book.assignments {
        columns.push(CsvColumn {
            header: assignment.title.clone(),
            assignment_id: Some(assignment.id.clone()),
            criterion_id: None,
        });
        if let Some(rubric) = rubric_for_assignment(&conn, &assignment.id)? {
            for criterion in &rubric.criteria {
                columns.push(CsvColumn {
                    header: format!("{} / {}", assignment.title, criterion.name),
                    assignment_id: Some(assignment.id.clone()),
                    criterion_id: Some(criterion.id.clone()),
                });
            }
        }
    }

    let mut lines = vec![columns
        .iter()
        .map(|c| escape_csv(&c.header))
        .collect::<Vec<_>>()
        .join(",")];

    for student in &book.students {
        let mut cells = vec![escape_csv(&student.student_name)];
        for column in columns.iter().skip(1) {
            let Some(assignment_id) = column.assignment_id.as_ref() else {
                cells.push(String::new());
                continue;
            };
            if column.criterion_id.is_none() {
                let points = student
                    .grades
                    .get(assignment_id)
                    .and_then(|g| g.points)
                    .map(|p| p.to_string())
                    .unwrap_or_default();
                cells.push(points);
            } else {
                let rubric_json: Option<String> = conn
                    .query_row(
                        "SELECT rubric_scores_json FROM grades WHERE assignment_id = ?1 AND student_id = ?2",
                        params![assignment_id, student.student_id],
                        |row| row.get(0),
                    )
                    .ok();
                cells.push(criterion_points(
                    rubric_json.as_deref(),
                    column.criterion_id.as_ref().unwrap(),
                ));
            }
        }
        lines.push(cells.join(","));
    }

    Ok(lines.join("\n"))
}

fn criterion_points(rubric_json: Option<&str>, criterion_id: &str) -> String {
    let Some(json) = rubric_json.filter(|s| !s.is_empty()) else {
        return String::new();
    };
    let scores: Vec<RubricScoreInput> = serde_json::from_str(json).unwrap_or_default();
    scores
        .iter()
        .find(|s| s.criterion_id == criterion_id)
        .map(|s| s.points.to_string())
        .unwrap_or_default()
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
