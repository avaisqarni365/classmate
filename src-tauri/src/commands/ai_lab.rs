use crate::models::{AiLabDefinition, MaterialAiLab};
use rusqlite::Connection;
use serde::Serialize;

pub const DEFAULT_ARTIZAI_BASE_URL: &str = "https://artizai.uk";

fn lab(
    slug: &str,
    name: &str,
    description: &str,
    tools: &[&str],
) -> AiLabDefinition {
    AiLabDefinition {
        slug: slug.into(),
        name: name.into(),
        description: description.into(),
        tools: tools.iter().map(|t| (*t).to_string()).collect(),
    }
}

fn all_labs() -> Vec<AiLabDefinition> {
    vec![
        lab(
            "science",
            "Science Lab",
            "Explore biology, chemistry, and physics with AI-guided inquiry.",
            &["Claude", "AlphaFold", "Roboflow"],
        ),
        lab(
            "maths",
            "Maths Lab",
            "Step-by-step reasoning, visualisations, and practice generation.",
            &["Claude", "Wolfram", "Desmos"],
        ),
        lab(
            "arts",
            "Arts Lab",
            "Creative projects across visual arts, media, and design.",
            &["Midjourney", "Claude", "Canva"],
        ),
        lab(
            "social-science",
            "Social Science Lab",
            "Economics, civics, psychology, and society through AI projects.",
            &["Claude", "Data tools"],
        ),
        lab(
            "languages",
            "Languages Lab",
            "Reading, writing, and conversation practice in any language.",
            &["Claude", "Speech tools"],
        ),
        lab(
            "humanities",
            "Humanities Lab",
            "History, literature, ethics, and critical thinking sprints.",
            &["Claude", "Research tools"],
        ),
        lab(
            "making-engineering",
            "Making & Engineering Lab",
            "Build, code, and engineer portfolio-ready prototypes.",
            &["Fusion 360", "Claude", "Roboflow"],
        ),
    ]
}

#[derive(Serialize)]
pub struct ArtizAiConfig {
    pub base_url: String,
    pub labs: Vec<AiLabDefinition>,
}

#[tauri::command]
pub fn list_ai_labs() -> Vec<AiLabDefinition> {
    all_labs()
}

#[tauri::command]
pub fn get_artizai_config(state: tauri::State<'_, crate::AppState>) -> Result<ArtizAiConfig, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(artizai_config(&conn))
}

#[tauri::command]
pub fn save_artizai_base_url(
    state: tauri::State<'_, crate::AppState>,
    base_url: String,
) -> Result<ArtizAiConfig, String> {
    let trimmed = base_url.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        return Err("ARTIZAI base URL is required".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::commands::sync::set_setting_work(&conn, "artizai_base_url", &trimmed)?;
    Ok(artizai_config(&conn))
}

#[tauri::command]
pub fn resolve_material_ai_lab(
    state: tauri::State<'_, crate::AppState>,
    course_code: String,
    course_title: String,
    material_title: String,
    subjects: Option<Vec<String>>,
) -> Result<MaterialAiLab, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(build_material_ai_lab(
        &conn,
        &course_code,
        &course_title,
        &material_title,
        subjects.as_deref(),
        None,
    ))
}

pub fn read_artizai_base_url(conn: &Connection) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'artizai_base_url'",
        [],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_else(|_| DEFAULT_ARTIZAI_BASE_URL.to_string())
}

pub fn build_material_ai_lab(
    conn: &Connection,
    course_code: &str,
    course_title: &str,
    material_title: &str,
    subjects: Option<&[String]>,
    openstax_slug: Option<String>,
) -> MaterialAiLab {
    let base_url = read_artizai_base_url(conn);
    let haystack = build_haystack(course_code, course_title, material_title, subjects);
    let lab = resolve_lab(&haystack);
    let mut url = build_lab_url(
        &base_url,
        &lab.slug,
        course_code,
        course_title,
        material_title,
    );
    if let Some(slug) = openstax_slug {
        url.push_str("&openstax=");
        url.push_str(&encode_component(&slug));
    }
    MaterialAiLab {
        lab,
        url: url.clone(),
        embed_url: url,
        activities: lab_activities(&haystack),
    }
}

fn build_haystack(
    course_code: &str,
    course_title: &str,
    material_title: &str,
    subjects: Option<&[String]>,
) -> String {
    let mut parts = vec![
        course_code.to_lowercase(),
        course_title.to_lowercase(),
        material_title.to_lowercase(),
    ];
    if let Some(list) = subjects {
        for subject in list {
            parts.push(subject.to_lowercase());
        }
    }
    parts.join(" ")
}

fn resolve_lab(haystack: &str) -> AiLabDefinition {
    let rules: [(&str, &[&str]); 7] = [
        (
            "making-engineering",
            &[
                "engineer", "coding", "code", "python", "java", "robot", "cs ", "computer",
                "program", "software", "tech", "cad", "maker", "build",
            ],
        ),
        (
            "maths",
            &[
                "math", "algebra", "calculus", "geometry", "trigonometry", "statistics",
                "precal", "arith",
            ],
        ),
        (
            "science",
            &[
                "science", "biology", "chemistry", "physics", "anatomy", "nursing",
                "physio", "ecology", "genetic",
            ],
        ),
        (
            "languages",
            &[
                "language", "english", "spanish", "french", "german", "mandarin", "literacy",
                "writing", "grammar", "esl",
            ],
        ),
        (
            "humanities",
            &[
                "history", "humanities", "philosophy", "literature", "religion", "ethics",
                "civilization",
            ],
        ),
        (
            "social-science",
            &[
                "social", "economics", "psychology", "sociology", "civics", "government",
                "politic", "business", "finance", "account",
            ],
        ),
        (
            "arts",
            &[
                "art", "design", "music", "drama", "theatre", "creative", "media", "visual",
            ],
        ),
    ];

    for (slug, keywords) in rules {
        if keywords.iter().any(|kw| haystack.contains(kw)) {
            return lab_by_slug(slug);
        }
    }
    lab_by_slug("science")
}

fn lab_by_slug(slug: &str) -> AiLabDefinition {
    all_labs()
        .into_iter()
        .find(|lab| lab.slug == slug)
        .unwrap_or_else(|| lab("science", "Science Lab", "Explore biology, chemistry, and physics with AI-guided inquiry.", &["Claude", "AlphaFold", "Roboflow"]))
}

fn lab_activities(haystack: &str) -> Vec<String> {
    let slug = resolve_lab(haystack).slug;
    match slug.as_str() {
        "maths" => vec![
            "Explain step-by-step".into(),
            "Generate practice problems".into(),
            "Visualise this topic".into(),
        ],
        "arts" => vec![
            "Brainstorm creative directions".into(),
            "Critique my draft work".into(),
            "Plan a portfolio piece".into(),
        ],
        "social-science" => vec![
            "Summarise key debates".into(),
            "Case study analysis".into(),
            "Generate discussion questions".into(),
        ],
        "languages" => vec![
            "Practice conversation".into(),
            "Check my writing".into(),
            "Build vocabulary list".into(),
        ],
        "humanities" => vec![
            "Timeline this topic".into(),
            "Compare primary sources".into(),
            "Essay outline helper".into(),
        ],
        "making-engineering" => vec![
            "Design a build plan".into(),
            "Debug my approach".into(),
            "Generate project brief".into(),
        ],
        _ => vec![
            "Explain this concept".into(),
            "Design a mini lab".into(),
            "Generate quiz questions".into(),
        ],
    }
}

pub fn build_lab_url(
    base_url: &str,
    lab_slug: &str,
    course_code: &str,
    course_title: &str,
    material_title: &str,
) -> String {
    format!(
        "{}/lab/{}?source=classmate&course={}&course_title={}&topic={}",
        base_url.trim_end_matches('/'),
        lab_slug,
        encode_component(course_code),
        encode_component(course_title),
        encode_component(material_title),
    )
}

fn encode_component(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            b' ' => "+".to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

pub fn artizai_config(conn: &Connection) -> ArtizAiConfig {
    ArtizAiConfig {
        base_url: read_artizai_base_url(conn),
        labs: all_labs(),
    }
}
