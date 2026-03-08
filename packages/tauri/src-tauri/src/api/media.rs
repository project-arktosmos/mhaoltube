use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_content))
        .route("/{youtube_id}", get(get_content))
}

#[derive(Serialize)]
struct YouTubeContentResponse {
    #[serde(rename = "youtubeId")]
    youtube_id: String,
    title: String,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: Option<String>,
    #[serde(rename = "durationSeconds")]
    duration_seconds: Option<i64>,
    #[serde(rename = "channelName")]
    channel_name: Option<String>,
    #[serde(rename = "channelId")]
    channel_id: Option<String>,
    #[serde(rename = "hasVideo")]
    has_video: bool,
    #[serde(rename = "hasAudio")]
    has_audio: bool,
    #[serde(rename = "createdAt")]
    created_at: String,
}

fn map_content(row: crate::db::repo::youtube_content::YouTubeContentRow) -> YouTubeContentResponse {
    YouTubeContentResponse {
        youtube_id: row.youtube_id,
        title: row.title,
        thumbnail_url: row.thumbnail_url,
        duration_seconds: row.duration_seconds,
        channel_name: row.channel_name,
        channel_id: row.channel_id,
        has_video: row.video_path.is_some(),
        has_audio: row.audio_path.is_some(),
        created_at: row.created_at,
    }
}

async fn get_all_content(State(state): State<AppState>) -> impl IntoResponse {
    let rows = state.youtube_content.get_all();
    let content: Vec<YouTubeContentResponse> = rows.into_iter().map(map_content).collect();
    Json(content)
}

async fn get_content(
    State(state): State<AppState>,
    Path(youtube_id): Path<String>,
) -> impl IntoResponse {
    match state.youtube_content.get(&youtube_id) {
        Some(row) => Json(map_content(row)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
