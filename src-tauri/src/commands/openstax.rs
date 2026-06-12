use crate::models::OpenStaxBook;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::sync::Mutex;

const OPENSTAX_BOOKS_URL: &str = "https://openstax.org/apps/cms/api/books/?format=json";
const CACHE_TTL: Duration = Duration::hours(6);

static BOOK_CACHE: Mutex<Option<(DateTime<Utc>, Vec<OpenStaxBook>)>> = Mutex::new(None);

#[derive(Deserialize)]
struct CmsBooksResponse {
    books: Vec<CmsBook>,
}

#[derive(Deserialize)]
struct CmsBook {
    title: String,
    slug: String,
    subjects: Vec<String>,
    book_state: String,
    webview_rex_link: Option<String>,
    high_resolution_pdf_url: Option<String>,
}

#[tauri::command]
pub fn list_openstax_books(subject: Option<String>) -> Result<Vec<OpenStaxBook>, String> {
    let books = fetch_openstax_books()?;
    let filter = subject
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());
    Ok(if let Some(query) = filter {
        books
            .into_iter()
            .filter(|book| {
                book.subjects.iter().any(|s| s.to_lowercase().contains(&query))
                    || book.title.to_lowercase().contains(&query)
            })
            .collect()
    } else {
        books
    })
}

pub fn fetch_openstax_books() -> Result<Vec<OpenStaxBook>, String> {
    if let Ok(guard) = BOOK_CACHE.lock() {
        if let Some((fetched_at, books)) = guard.as_ref() {
            if Utc::now().signed_duration_since(*fetched_at) < CACHE_TTL {
                return Ok(books.clone());
            }
        }
    }

    let response = ureq::get(OPENSTAX_BOOKS_URL)
        .call()
        .map_err(|e| format!("Could not reach OpenStax catalog: {e}"))?;
    if response.status() != 200 {
        return Err(format!("OpenStax catalog returned HTTP {}", response.status()));
    }

    let payload: CmsBooksResponse = response
        .into_json()
        .map_err(|e| format!("Invalid OpenStax catalog response: {e}"))?;

    let books = payload
        .books
        .into_iter()
        .filter(|book| {
            matches!(
                book.book_state.as_str(),
                "live" | "new_edition_available"
            )
        })
        .filter_map(normalize_book)
        .collect::<Vec<_>>();

    if let Ok(mut guard) = BOOK_CACHE.lock() {
        *guard = Some((Utc::now(), books.clone()));
    }

    Ok(books)
}

fn normalize_book(book: CmsBook) -> Option<OpenStaxBook> {
    let slug = book
        .slug
        .rsplit('/')
        .next()
        .unwrap_or(book.slug.as_str())
        .to_string();
    let read_url = book
        .webview_rex_link
        .filter(|url| !url.trim().is_empty())
        .unwrap_or_else(|| format!("https://openstax.org/books/{slug}/pages/1-introduction"));
    Some(OpenStaxBook {
        slug,
        title: book.title,
        subjects: book.subjects,
        read_url,
        pdf_url: book
            .high_resolution_pdf_url
            .filter(|url| !url.trim().is_empty()),
    })
}
