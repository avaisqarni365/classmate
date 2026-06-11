use crate::models::{AssignmentRubric, RubricCriterion, SaveAssignmentRubricInput};
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn get_assignment_rubric(
    state: State<'_, AppState>,
    assignment_id: String,
) -> Result<Option<AssignmentRubric>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    rubric_for_assignment(&conn, &assignment_id)
}

#[tauri::command]
pub fn save_assignment_rubric(
    state: State<'_, AppState>,
    input: SaveAssignmentRubricInput,
) -> Result<AssignmentRubric, String> {
    if input.criteria.is_empty() {
        return Err("Add at least one rubric criterion".into());
    }
    let mut total = 0.0;
    let mut criteria = Vec::new();
    for item in input.criteria {
        if item.name.trim().is_empty() {
            return Err("Each criterion needs a name".into());
        }
        if item.max_points <= 0.0 {
            return Err("Criterion points must be greater than zero".into());
        }
        total += item.max_points;
        criteria.push(RubricCriterion {
            id: item.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: item.name.trim().to_string(),
            description: item.description.filter(|s| !s.trim().is_empty()),
            max_points: item.max_points,
        });
    }

    let now = Utc::now().to_rfc3339();
    let criteria_json = serde_json::to_string(&criteria).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT course_id FROM assignments WHERE id = ?1",
        params![input.assignment_id],
        |row| row.get::<_, String>(0),
    )
    .map_err(|_| "Assignment not found".to_string())?;

    conn.execute(
        "INSERT INTO assignment_rubrics (assignment_id, criteria_json, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT(assignment_id) DO UPDATE SET criteria_json = excluded.criteria_json, updated_at = excluded.updated_at",
        params![input.assignment_id, criteria_json, now],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE assignments SET max_points = ?1 WHERE id = ?2",
        params![total, input.assignment_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(AssignmentRubric {
        assignment_id: input.assignment_id,
        criteria,
        updated_at: now,
    })
}

pub fn rubric_for_assignment(
    conn: &rusqlite::Connection,
    assignment_id: &str,
) -> Result<Option<AssignmentRubric>, String> {
    let row = conn.query_row(
        "SELECT assignment_id, criteria_json, updated_at FROM assignment_rubrics WHERE assignment_id = ?1",
        params![assignment_id],
        |row| {
            let criteria_json: String = row.get(1)?;
            let criteria: Vec<RubricCriterion> =
                serde_json::from_str(&criteria_json).unwrap_or_default();
            Ok(AssignmentRubric {
                assignment_id: row.get(0)?,
                criteria,
                updated_at: row.get(2)?,
            })
        },
    );
    match row {
        Ok(rubric) => Ok(Some(rubric)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn rubric_scores_display(
    conn: &rusqlite::Connection,
    assignment_id: &str,
    rubric_scores_json: Option<&str>,
) -> Result<Option<Vec<crate::models::RubricScoreDisplay>>, String> {
    let Some(json) = rubric_scores_json.filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let scores: Vec<crate::models::RubricScoreInput> =
        serde_json::from_str(json).map_err(|e| e.to_string())?;
    let Some(rubric) = rubric_for_assignment(conn, assignment_id)? else {
        return Ok(None);
    };
    let mut out = Vec::new();
    for score in scores {
        if let Some(criterion) = rubric.criteria.iter().find(|c| c.id == score.criterion_id) {
            out.push(crate::models::RubricScoreDisplay {
                criterion_id: criterion.id.clone(),
                name: criterion.name.clone(),
                max_points: criterion.max_points,
                points: score.points,
            });
        }
    }
    if out.is_empty() {
        Ok(None)
    } else {
        Ok(Some(out))
    }
}
