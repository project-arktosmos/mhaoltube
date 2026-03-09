use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
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
        audio: list_dir(&base.join("audio").join(".cache")),
        video: list_dir(&base.join("video").join(".cache")),
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
    headers: HeaderMap,
) -> impl IntoResponse {
    let content = match state.youtube_content.get(&youtube_id) {
        Some(c) => c,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let path_str = match content.video_path {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());

    stream_file(&path_str, range.as_deref()).await
}

async fn stream_audio(
    State(state): State<AppState>,
    Path(youtube_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let content = match state.youtube_content.get(&youtube_id) {
        Some(c) => c,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let path_str = match content.audio_path {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());

    stream_file(&path_str, range.as_deref()).await
}

pub(crate) async fn stream_file(path_str: &str, range_header: Option<&str>) -> axum::response::Response {
    let path = std::path::Path::new(path_str);

    let file = match tokio::fs::File::open(path).await {
        Ok(f) => f,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let file_size = match file.metadata().await {
        Ok(m) => m.len(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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

    if let Some(range_str) = range_header {
        if let Some(range_val) = range_str.strip_prefix("bytes=") {
            let parts: Vec<&str> = range_val.splitn(2, '-').collect();
            if parts.len() == 2 {
                let start: u64 = parts[0].parse().unwrap_or(0);
                let end: u64 = parts[1]
                    .parse()
                    .unwrap_or_else(|_| file_size.saturating_sub(1))
                    .min(file_size.saturating_sub(1));

                if start >= file_size || start > end {
                    return Response::builder()
                        .status(StatusCode::RANGE_NOT_SATISFIABLE)
                        .header(header::CONTENT_RANGE, format!("bytes */{}", file_size))
                        .body(Body::empty())
                        .unwrap();
                }

                let length = end - start + 1;
                let mut file = file;

                if file.seek(std::io::SeekFrom::Start(start)).await.is_err() {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                let limited = file.take(length);
                let stream = ReaderStream::new(limited);

                return Response::builder()
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(
                        header::CONTENT_RANGE,
                        format!("bytes {}-{}/{}", start, end, file_size),
                    )
                    .header(header::CONTENT_LENGTH, length.to_string())
                    .header(header::ACCEPT_RANGES, "bytes")
                    .body(Body::from_stream(stream))
                    .unwrap();
            }
        }
    }

    // No Range header — stream entire file
    let stream = ReaderStream::new(file);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::from_stream(stream))
        .unwrap()
}
