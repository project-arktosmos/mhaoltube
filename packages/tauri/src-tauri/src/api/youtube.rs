use crate::db::DbPool;
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oembed", get(oembed))
        .route("/channel-feed", get(channel_feed))
}

/// Fetch YouTube oEmbed data for a video ID, using the cache if available.
/// On cache miss, fetches from YouTube and caches the result.
pub async fn fetch_and_cache_oembed(
    db: &DbPool,
    video_id: &str,
) -> Option<serde_json::Value> {
    // Check cache
    {
        let conn = db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM youtube_videos WHERE video_id = ?1",
            rusqlite::params![video_id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Some(parsed);
            }
        }
    }

    let url = format!(
        "https://www.youtube.com/oembed?url=https://www.youtube.com/watch?v={}&format=json",
        video_id
    );

    let resp = reqwest::get(&url).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let data = resp.json::<serde_json::Value>().await.ok()?;
    let data_str = serde_json::to_string(&data).unwrap_or_default();
    let conn = db.lock();
    let _ = conn.execute(
        "INSERT INTO youtube_videos (video_id, data) VALUES (?1, ?2)
         ON CONFLICT(video_id) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
        rusqlite::params![video_id, data_str],
    );

    Some(data)
}

#[derive(Deserialize)]
struct OembedQuery {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

async fn oembed(
    State(state): State<AppState>,
    Query(query): Query<OembedQuery>,
) -> impl IntoResponse {
    let video_id = match &query.video_id {
        Some(id) if id.len() == 11 => id,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing or invalid videoId parameter" })),
            )
                .into_response()
        }
    };

    match fetch_and_cache_oembed(&state.db, video_id).await {
        Some(data) => Json(data).into_response(),
        None => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": "YouTube oEmbed API unavailable" })),
        )
            .into_response(),
    }
}

// ===== Channel Feed (InnerTube Browse) =====

const INNERTUBE_BROWSE_URL: &str = "https://www.youtube.com/youtubei/v1/browse";

/// Params value that selects the "Videos" tab sorted by latest
const CHANNEL_VIDEOS_PARAMS: &str = "EgZ2aWRlb3PyBgQKAjoA";

#[derive(Serialize)]
struct ChannelFeedVideo {
    #[serde(rename = "videoId")]
    video_id: String,
    title: String,
    thumbnail: String,
    duration: i64,
    #[serde(rename = "durationText")]
    duration_text: String,
    views: i64,
    #[serde(rename = "viewsText")]
    views_text: String,
    #[serde(rename = "publishedText")]
    published_text: String,
}

#[derive(Serialize)]
struct ChannelFeedResponse {
    #[serde(rename = "channelId")]
    channel_id: String,
    videos: Vec<ChannelFeedVideo>,
    continuation: Option<String>,
}

#[derive(Deserialize)]
struct ChannelFeedQuery {
    #[serde(rename = "channelId")]
    channel_id: String,
    continuation: Option<String>,
}

async fn channel_feed(Query(query): Query<ChannelFeedQuery>) -> impl IntoResponse {
    if query.channel_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing channelId parameter" })),
        )
            .into_response();
    }

    let mut body = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20240101.00.00",
                "hl": "en",
                "gl": "US"
            }
        }
    });

    if let Some(token) = &query.continuation {
        body["continuation"] = serde_json::Value::String(token.clone());
    } else {
        body["browseId"] = serde_json::Value::String(query.channel_id.clone());
        body["params"] = serde_json::Value::String(CHANNEL_VIDEOS_PARAMS.to_string());
    }

    let client = reqwest::Client::new();
    let resp = match client
        .post(INNERTUBE_BROWSE_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if !resp.status().is_success() {
        return (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("YouTube API error: {}", resp.status()) })),
        )
            .into_response();
    }

    let data: serde_json::Value = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let (videos, continuation) = parse_channel_browse(&data, query.continuation.is_some());

    Json(ChannelFeedResponse {
        channel_id: query.channel_id,
        videos,
        continuation,
    })
    .into_response()
}

fn parse_channel_browse(
    data: &serde_json::Value,
    is_continuation: bool,
) -> (Vec<ChannelFeedVideo>, Option<String>) {
    let mut videos = Vec::new();
    let mut continuation = None;

    let items: Vec<&serde_json::Value> = if is_continuation {
        // Continuation: onResponseReceivedActions[].appendContinuationItemsAction.continuationItems
        data.get("onResponseReceivedActions")
            .and_then(|a| a.as_array())
            .map(|actions| {
                let mut items = Vec::new();
                for action in actions {
                    if let Some(ci) = action
                        .pointer("/appendContinuationItemsAction/continuationItems")
                        .and_then(|c| c.as_array())
                    {
                        items.extend(ci.iter());
                    }
                }
                items
            })
            .unwrap_or_default()
    } else {
        // Initial: contents.twoColumnBrowseResultsRenderer.tabs[].tabRenderer.content
        //          .richGridRenderer.contents
        let tabs = data.pointer(
            "/contents/twoColumnBrowseResultsRenderer/tabs",
        );
        tabs.and_then(|t| t.as_array())
            .and_then(|tabs| {
                tabs.iter().find(|tab| {
                    tab.pointer("/tabRenderer/selected")
                        .and_then(|s| s.as_bool())
                        .unwrap_or(false)
                })
            })
            .and_then(|tab| {
                tab.pointer("/tabRenderer/content/richGridRenderer/contents")
                    .and_then(|c| c.as_array())
            })
            .map(|arr| arr.iter().collect())
            .unwrap_or_default()
    };

    for item in &items {
        // richItemRenderer wraps the videoRenderer
        if let Some(video) = item.pointer("/richItemRenderer/content/videoRenderer") {
            if let Some(v) = parse_browse_video(video) {
                videos.push(v);
            }
        }

        // Continuation token
        if let Some(token) = item
            .pointer("/continuationItemRenderer/continuationEndpoint/continuationCommand/token")
        {
            continuation = token.as_str().map(String::from);
        }
    }

    (videos, continuation)
}

fn parse_browse_video(v: &serde_json::Value) -> Option<ChannelFeedVideo> {
    let video_id = v.get("videoId")?.as_str()?;

    let title = v
        .pointer("/title/runs/0/text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let thumbnail = v
        .pointer("/thumbnail/thumbnails")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    let duration_text = v
        .pointer("/lengthText/simpleText")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    let duration = parse_duration_text(&duration_text);

    let views_text = v
        .pointer("/viewCountText/simpleText")
        .or_else(|| v.pointer("/viewCountText/runs/0/text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    let views = parse_view_count_text(&views_text);

    let published_text = v
        .pointer("/publishedTimeText/simpleText")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    Some(ChannelFeedVideo {
        video_id: video_id.to_string(),
        title,
        thumbnail,
        duration,
        duration_text,
        views,
        views_text,
        published_text,
    })
}

fn parse_duration_text(text: &str) -> i64 {
    let parts: Vec<&str> = text.split(':').collect();
    match parts.len() {
        3 => {
            let h: i64 = parts[0].parse().unwrap_or(0);
            let m: i64 = parts[1].parse().unwrap_or(0);
            let s: i64 = parts[2].parse().unwrap_or(0);
            h * 3600 + m * 60 + s
        }
        2 => {
            let m: i64 = parts[0].parse().unwrap_or(0);
            let s: i64 = parts[1].parse().unwrap_or(0);
            m * 60 + s
        }
        1 => parts[0].parse().unwrap_or(0),
        _ => 0,
    }
}

fn parse_view_count_text(text: &str) -> i64 {
    let cleaned = text
        .replace(" views", "")
        .replace(" view", "")
        .replace(',', "");
    cleaned.parse().unwrap_or(0)
}
