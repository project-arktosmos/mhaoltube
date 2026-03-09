use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_default_library))
        .route("/fs", get(list_library_fs))
        .route(
            "/content/{youtube_id}/stream/video",
            get(stream_video),
        )
        .route(
            "/content/{youtube_id}/stream/audio",
            get(stream_audio),
        )
}

#[derive(Serialize)]
struct MappedLibrary {
    id: String,
    name: String,
    path: String,
    #[serde(rename = "dateAdded")]
    date_added: i64,
}

#[derive(Serialize)]
struct FsEntry {
    name: String,
    size: u64,
}

#[derive(Serialize)]
struct LibraryFs {
    path: String,
    audio: Vec<FsEntry>,
    video: Vec<FsEntry>,
}

fn list_dir(dir: &std::path::Path) -> Vec<FsEntry> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<FsEntry> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_str()?.to_string();
            if name.starts_with('.') {
                return None;
            }
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            Some(FsEntry { name, size })
        })
        .collect();
    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

async fn list_library_fs(State(state): State<AppState>) -> impl IntoResponse {
    let library = match state.libraries.get(crate::AppState::DEFAULT_LIBRARY_ID) {
        Some(lib) => lib,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Library not found" })),
            )
                .into_response()
        }
    };
    let base = std::path::PathBuf::from(&library.path);
    Json(LibraryFs {
        path: library.path,
        audio: list_dir(&base.join("audio")),
        video: list_dir(&base.join("video")),
    })
    .into_response()
}

async fn get_default_library(State(state): State<AppState>) -> impl IntoResponse {
    match state.libraries.get(crate::AppState::DEFAULT_LIBRARY_ID) {
        Some(row) => Json(MappedLibrary {
            id: row.id,
            name: row.name,
            path: row.path,
            date_added: row.date_added,
        })
        .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn stream_video(
    State(state): State<AppState>,
    Path(youtube_id): Path<String>,
) -> impl IntoResponse {
    let content = match state.youtube_content.get(&youtube_id) {
        Some(c) => c,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let path_str = match content.video_path {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    stream_file(&path_str).await
}

async fn stream_audio(
    State(state): State<AppState>,
    Path(youtube_id): Path<String>,
) -> impl IntoResponse {
    let content = match state.youtube_content.get(&youtube_id) {
        Some(c) => c,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let path_str = match content.audio_path {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    stream_file(&path_str).await
}

async fn stream_file(path_str: &str) -> axum::response::Response {
    let path = std::path::Path::new(path_str);
    let bytes = match tokio::fs::read(path).await {
        Ok(b) => b,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let content_type = match path.extension().and_then(|e| e.to_str()) {
        Some("mp4") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("mp3") => "audio/mpeg",
        Some("flac") => "audio/flac",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("opus") => "audio/opus",
        Some("aac") => "audio/aac",
        _ => "application/octet-stream",
    };

    ([(header::CONTENT_TYPE, content_type)], bytes).into_response()
}
