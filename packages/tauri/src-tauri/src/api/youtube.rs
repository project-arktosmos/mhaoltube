use crate::db::DbPool;
use crate::AppState;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oembed", get(oembed))
        .route("/channel-feed", get(channel_feed))
        .route("/channel-rss", get(channel_rss))
        .route("/channel-meta", get(channel_meta))
        .route("/image-proxy", get(image_proxy))
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

// ===== Channel RSS Feed =====

#[derive(Serialize)]
struct RssVideo {
    #[serde(rename = "videoId")]
    video_id: String,
    title: String,
    published: String,
    #[serde(rename = "publishedText")]
    published_text: String,
    thumbnail: String,
    views: i64,
    #[serde(rename = "viewsText")]
    views_text: String,
}

#[derive(Serialize)]
struct RssFeedResponse {
    #[serde(rename = "channelId")]
    channel_id: String,
    #[serde(rename = "channelName")]
    channel_name: String,
    videos: Vec<RssVideo>,
}

#[derive(Deserialize)]
struct RssFeedQuery {
    handle: String,
}

async fn channel_rss(Query(query): Query<RssFeedQuery>) -> impl IntoResponse {
    if query.handle.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing handle parameter" })),
        )
            .into_response();
    }

    // Resolve YouTube handle to real channel ID by fetching the channel page
    let channel_id = match resolve_channel_id(&query.handle).await {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    let url = format!(
        "https://www.youtube.com/feeds/videos.xml?channel_id={}",
        channel_id
    );

    let resp = match reqwest::get(&url).await {
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
            Json(serde_json::json!({ "error": format!("YouTube RSS error: {}", resp.status()) })),
        )
            .into_response();
    }

    let xml = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let (channel_name, videos) = parse_rss_feed(&xml);

    Json(RssFeedResponse {
        channel_id,
        channel_name,
        videos,
    })
    .into_response()
}

/// Fetch a YouTube channel page HTML by handle.
async fn fetch_channel_page(handle: &str) -> Result<String, String> {
    let page_url = format!("https://www.youtube.com/@{}", handle);
    let client = reqwest::Client::new();
    let resp = client
        .get(&page_url)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch channel page: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Channel page returned {}", resp.status()));
    }

    resp.text()
        .await
        .map_err(|e| format!("Failed to read channel page: {}", e))
}

/// Resolve a YouTube handle to the real UC... channel ID from page HTML.
async fn resolve_channel_id(handle: &str) -> Result<String, String> {
    let html = fetch_channel_page(handle).await?;
    extract_channel_id(&html).ok_or_else(|| "Could not find channel ID in page".to_string())
}

fn extract_channel_id(html: &str) -> Option<String> {
    for pattern in &["\"externalId\":\"", "\"channelId\":\""] {
        if let Some(start) = html.find(pattern) {
            let rest = &html[start + pattern.len()..];
            if let Some(end) = rest.find('"') {
                let id = &rest[..end];
                if id.starts_with("UC") {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

// ===== Channel Metadata =====

#[derive(Serialize)]
struct ChannelMeta {
    #[serde(rename = "channelId")]
    channel_id: String,
    avatar: String,
    description: String,
    #[serde(rename = "subscriberText")]
    subscriber_text: String,
}

#[derive(Deserialize)]
struct ChannelMetaQuery {
    handle: String,
}

async fn channel_meta(
    State(state): State<AppState>,
    Query(query): Query<ChannelMetaQuery>,
) -> impl IntoResponse {
    if query.handle.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing handle parameter" })),
        )
            .into_response();
    }

    // Check if we already have cached metadata in the DB
    let cached = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT id, image_url, subscriber_text FROM youtube_channels WHERE handle = ?1",
            rusqlite::params![query.handle],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .ok()
    };

    if let Some((channel_id, Some(image_url), Some(subscriber_text))) = &cached {
        if !image_url.is_empty() && !subscriber_text.is_empty() {
            return Json(ChannelMeta {
                channel_id: channel_id.clone(),
                avatar: image_url.clone(),
                description: String::new(),
                subscriber_text: subscriber_text.clone(),
            })
            .into_response();
        }
    }

    // Not cached — fetch from YouTube
    let html = match fetch_channel_page(&query.handle).await {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    let channel_id = extract_channel_id(&html).unwrap_or_default();
    let avatar = extract_og_meta(&html, "og:image").unwrap_or_default();
    let description = extract_og_meta(&html, "og:description").unwrap_or_default();
    let subscriber_text = extract_subscriber_count(&html).unwrap_or_default();

    // Persist to DB if we have a matching channel row
    if !avatar.is_empty() || !subscriber_text.is_empty() {
        let conn = state.db.lock();
        let _ = conn.execute(
            "UPDATE youtube_channels SET image_url = ?1, subscriber_text = ?2 WHERE handle = ?3",
            rusqlite::params![avatar, subscriber_text, query.handle],
        );
    }

    Json(ChannelMeta {
        channel_id,
        avatar,
        description,
        subscriber_text,
    })
    .into_response()
}

/// Extract an Open Graph meta tag value from HTML.
fn extract_og_meta(html: &str, property: &str) -> Option<String> {
    // Look for <meta property="og:image" content="...">
    let pattern = format!("property=\"{}\"", property);
    let pos = html.find(&pattern)?;
    let region = &html[pos..];
    // Find content="..." attribute
    let content_start = region.find("content=\"")? + 9;
    let rest = &region[content_start..];
    let end = rest.find('"')?;
    Some(html_decode(&rest[..end]))
}

/// Extract subscriber count from the channel page HTML.
/// YouTube embeds this in several formats depending on the channel.
fn extract_subscriber_count(html: &str) -> Option<String> {
    // Strategy 1: "subscriberCountText":{"simpleText":"..."}
    if let Some(pos) = html.find("\"subscriberCountText\"") {
        let rest = &html[pos..];
        if let Some(st) = extract_nested_text(rest, "\"simpleText\":\"") {
            return Some(st);
        }
        if let Some(st) = extract_nested_text(rest, "\"label\":\"") {
            if st.contains("subscriber") {
                return Some(st);
            }
        }
    }

    // Strategy 2: "content":"NNK subscribers" inside metadataParts
    // This is the most common format for many channels
    {
        let needle = " subscribers\"";
        let mut search_from = 0;
        while let Some(pos) = html[search_from..].find(needle) {
            let abs_pos = search_from + pos;
            // Walk backwards to find "content":" before this
            let start = if abs_pos > 200 { abs_pos - 200 } else { 0 };
            let region = &html[start..abs_pos];
            if let Some(content_pos) = region.rfind("\"content\":\"") {
                let val_start = content_pos + "\"content\":\"".len();
                let val = &region[val_start..];
                let decoded = decode_json_unicode(val);
                let trimmed = decoded.trim();
                if !trimmed.is_empty() {
                    return Some(format!("{} subscribers", trimmed));
                }
            }
            search_from = abs_pos + needle.len();
        }
    }

    // Strategy 3: Unicode bidi isolate chars around "X subscribers"
    {
        let marker = " subscribers";
        let mut search_from = 0;
        while let Some(pos) = html[search_from..].find(marker) {
            let abs_pos = search_from + pos;
            let before = &html[..abs_pos];
            if let Some(bidi_start) = before.rfind('\u{2068}') {
                if abs_pos - bidi_start < 50 {
                    let count = &html[bidi_start + '\u{2068}'.len_utf8()..abs_pos];
                    let count = count.trim();
                    if !count.is_empty() {
                        return Some(format!("{} subscribers", count));
                    }
                }
            }
            search_from = abs_pos + marker.len();
        }
    }

    None
}

/// Extract a quoted string value near a JSON key.
fn extract_nested_text(haystack: &str, key: &str) -> Option<String> {
    let start = haystack.find(key)?;
    if start > 300 {
        return None; // too far, wrong match
    }
    let val_start = start + key.len();
    let rest = &haystack[val_start..];
    let end = rest.find('"')?;
    let raw = &rest[..end];
    // Decode Unicode escapes like \u0026, \u2068 etc.
    Some(decode_json_unicode(raw))
}

/// Decode JSON unicode escape sequences and HTML entities.
fn decode_json_unicode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                            continue;
                        }
                    }
                    result.push_str("\\u");
                    result.push_str(&hex);
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    html_decode(&result)
}

fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

// ===== Image Proxy =====

#[derive(Deserialize)]
struct ImageProxyQuery {
    url: String,
}

async fn image_proxy(Query(query): Query<ImageProxyQuery>) -> impl IntoResponse {
    // Only allow proxying YouTube image URLs
    let allowed = query.url.starts_with("https://yt3.ggpht.com/")
        || query.url.starts_with("https://yt3.googleusercontent.com/")
        || query.url.starts_with("https://i.ytimg.com/");
    if !allowed {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Only YouTube image URLs are allowed" })),
        )
            .into_response();
    }

    let resp = match reqwest::get(&query.url).await {
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
        return StatusCode::BAD_GATEWAY.into_response();
    }

    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    (
        [(header::CONTENT_TYPE, content_type),
         (header::CACHE_CONTROL, "public, max-age=86400".to_string())],
        Body::from(bytes),
    )
        .into_response()
}

fn parse_rss_feed(xml: &str) -> (String, Vec<RssVideo>) {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut videos = Vec::new();
    let mut channel_name = String::new();

    // Track current element path for context
    let mut in_entry = false;
    let mut current_tag = String::new();

    // Entry fields
    let mut video_id = String::new();
    let mut title = String::new();
    let mut published = String::new();
    let mut views: i64 = 0;
    let mut found_channel_name = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let local = tag_name.split(':').last().unwrap_or(&tag_name).to_string();

                if local == "entry" {
                    in_entry = true;
                    video_id.clear();
                    title.clear();
                    published.clear();
                    views = 0;
                } else if !in_entry && local == "name" && !found_channel_name {
                    current_tag = "channel_name".to_string();
                } else if in_entry {
                    match local.as_str() {
                        "videoId" => current_tag = "videoId".to_string(),
                        "title" => current_tag = "title".to_string(),
                        "published" => current_tag = "published".to_string(),
                        "statistics" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"views" {
                                    views = String::from_utf8_lossy(&attr.value)
                                        .parse()
                                        .unwrap_or(0);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                match current_tag.as_str() {
                    "channel_name" => {
                        channel_name = text;
                        found_channel_name = true;
                    }
                    "videoId" => video_id = text,
                    "title" => title = text,
                    "published" => published = text,
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::End(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let local = tag_name.split(':').last().unwrap_or(&tag_name);

                if local == "entry" && in_entry {
                    if !video_id.is_empty() {
                        let thumbnail = format!(
                            "https://i.ytimg.com/vi/{}/mqdefault.jpg",
                            video_id
                        );
                        let published_text = format_relative_date(&published);
                        let views_text = format_view_count(views);

                        videos.push(RssVideo {
                            video_id: video_id.clone(),
                            title: title.clone(),
                            published: published.clone(),
                            published_text,
                            thumbnail,
                            views,
                            views_text,
                        });
                    }
                    in_entry = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    (channel_name, videos)
}

fn format_relative_date(iso_date: &str) -> String {
    use chrono::{DateTime, Utc};

    let parsed = iso_date.parse::<DateTime<Utc>>();
    let dt = match parsed {
        Ok(d) => d,
        Err(_) => return iso_date.to_string(),
    };

    let now = Utc::now();
    let diff = now.signed_duration_since(dt);

    let days = diff.num_days();
    if days == 0 {
        let hours = diff.num_hours();
        if hours == 0 {
            let mins = diff.num_minutes();
            if mins <= 1 {
                return "just now".to_string();
            }
            return format!("{} minutes ago", mins);
        }
        return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
    }
    if days == 1 {
        return "1 day ago".to_string();
    }
    if days < 7 {
        return format!("{} days ago", days);
    }
    if days < 30 {
        let weeks = days / 7;
        return format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" });
    }
    if days < 365 {
        let months = days / 30;
        return format!("{} month{} ago", months, if months == 1 { "" } else { "s" });
    }
    let years = days / 365;
    format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
}

fn format_view_count(views: i64) -> String {
    if views >= 1_000_000_000 {
        format!("{:.1}B views", views as f64 / 1_000_000_000.0)
    } else if views >= 1_000_000 {
        format!("{:.1}M views", views as f64 / 1_000_000.0)
    } else if views >= 1_000 {
        format!("{:.1}K views", views as f64 / 1_000.0)
    } else {
        format!("{} views", views)
    }
}
