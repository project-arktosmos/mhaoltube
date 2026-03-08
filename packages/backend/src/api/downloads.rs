use crate::AppState;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_downloads))
}

#[derive(Serialize)]
struct Download {
    id: String,
    #[serde(rename = "type")]
    download_type: String,
    name: String,
    state: String,
    progress: f64,
    size: i64,
    #[serde(rename = "outputPath")]
    output_path: Option<String>,
    error: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(rename = "thumbnailUrl", skip_serializing_if = "Option::is_none")]
    thumbnail_url: Option<String>,
}

async fn get_downloads(State(state): State<AppState>) -> impl IntoResponse {
    let youtube_rows = state.youtube_downloads.get_all();

    let mut downloads: Vec<Download> = youtube_rows
        .into_iter()
        .map(|row| Download {
            id: row.download_id,
            download_type: "youtube".to_string(),
            name: row.title,
            state: row.state,
            progress: row.progress,
            size: row.total_bytes,
            output_path: row.output_path,
            error: row.error,
            created_at: row.created_at,
            updated_at: row.updated_at,
            url: Some(row.url),
            mode: Some(row.mode),
            format: Some(row.format),
            thumbnail_url: row.thumbnail_url,
        })
        .collect();

    downloads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Json(downloads)
}
