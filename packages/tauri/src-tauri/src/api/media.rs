use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_content))
        .route("/favorites", get(get_favorites))
        .route("/fill-durations", post(fill_durations))
        .route("/{youtube_id}", get(get_content))
        .route("/{youtube_id}/favorite", put(toggle_favorite))
        .route("/{youtube_id}/audio", delete(delete_audio))
        .route("/{youtube_id}/video", delete(delete_video))
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
    #[serde(rename = "videoSize")]
    video_size: Option<u64>,
    #[serde(rename = "audioSize")]
    audio_size: Option<u64>,
    #[serde(rename = "isFavorite")]
    is_favorite: bool,
    #[serde(rename = "favoritedAt")]
    favorited_at: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
}

fn file_size(path: Option<&str>) -> Option<u64> {
    std::fs::metadata(path?).ok().map(|m| m.len())
}

fn map_content(row: crate::db::repo::youtube_content::YouTubeContentRow) -> YouTubeContentResponse {
    YouTubeContentResponse {
        has_video: row.video_path.is_some(),
        has_audio: row.audio_path.is_some(),
        video_size: file_size(row.video_path.as_deref()),
        audio_size: file_size(row.audio_path.as_deref()),
        is_favorite: row.is_favorite,
        favorited_at: row.favorited_at,
        youtube_id: row.youtube_id,
        title: row.title,
        thumbnail_url: row.thumbnail_url,
        duration_seconds: row.duration_seconds,
        channel_name: row.channel_name,
        channel_id: row.channel_id,
        created_at: row.created_at,
    }
}

async fn get_all_content(State(state): State<AppState>) -> impl IntoResponse {
    let rows = state.youtube_content.get_all();
    let content: Vec<YouTubeContentResponse> = rows.into_iter().map(map_content).collect();
    Json(content)
}

async fn get_favorites(State(state): State<AppState>) -> impl IntoResponse {
    let rows = state.youtube_content.get_favorites();
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

async fn toggle_favorite(
    State(state): State<AppState>,
    Path(youtube_id): Path<String>,
) -> impl IntoResponse {
    if state.youtube_content.get(&youtube_id).is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let is_favorite = state.youtube_content.toggle_favorite(&youtube_id);
    Json(serde_json::json!({ "isFavorite": is_favorite })).into_response()
}

async fn delete_audio(
    State(state): State<AppState>,
    Path(youtube_id): Path<String>,
) -> impl IntoResponse {
    let content = match state.youtube_content.get(&youtube_id) {
        Some(c) => c,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    if let Some(path) = &content.audio_path {
        let _ = std::fs::remove_file(path);
    }

    state.youtube_content.clear_audio_path(&youtube_id);

    // If no files remain, remove the DB row entirely
    if let Some(updated) = state.youtube_content.get(&youtube_id) {
        if updated.video_path.is_none() && updated.audio_path.is_none() {
            state.youtube_content.delete(&youtube_id);
        }
    }

    StatusCode::OK.into_response()
}

async fn delete_video(
    State(state): State<AppState>,
    Path(youtube_id): Path<String>,
) -> impl IntoResponse {
    let content = match state.youtube_content.get(&youtube_id) {
        Some(c) => c,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    if let Some(path) = &content.video_path {
        let _ = std::fs::remove_file(path);
    }

    state.youtube_content.clear_video_path(&youtube_id);

    // If no files remain, remove the DB row entirely
    if let Some(updated) = state.youtube_content.get(&youtube_id) {
        if updated.video_path.is_none() && updated.audio_path.is_none() {
            state.youtube_content.delete(&youtube_id);
        }
    }

    StatusCode::OK.into_response()
}

#[derive(Serialize)]
struct FilledDuration {
    #[serde(rename = "youtubeId")]
    youtube_id: String,
    #[serde(rename = "durationSeconds")]
    duration_seconds: i64,
}

async fn fill_durations(State(state): State<AppState>) -> impl IntoResponse {
    let ids = state.youtube_content.get_ids_missing_duration();
    if ids.is_empty() {
        return Json(Vec::<FilledDuration>::new()).into_response();
    }

    let client = reqwest::Client::new();
    let mut filled: Vec<FilledDuration> = Vec::new();

    for video_id in &ids {
        let body = serde_json::json!({
            "videoId": video_id,
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20260301.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            }
        });

        let result = client
            .post("https://www.youtube.com/youtubei/v1/player")
            .header("Content-Type", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36")
            .json(&body)
            .send()
            .await;

        let duration = match result {
            Ok(resp) if resp.status().is_success() => resp
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|data| {
                    data.pointer("/videoDetails/lengthSeconds")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i64>().ok())
                        .filter(|&d| d > 0)
                }),
            _ => None,
        };

        if let Some(secs) = duration {
            state.youtube_content.update_duration(video_id, secs);
            filled.push(FilledDuration {
                youtube_id: video_id.clone(),
                duration_seconds: secs,
            });
        }
    }

    Json(filled).into_response()
}
