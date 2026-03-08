#[cfg(feature = "embed-frontend")]
pub mod frontend;
pub mod database;
pub mod downloads;
pub mod libraries;
pub mod media;
pub mod media_lists;
pub mod youtube;
pub mod youtube_search;
#[cfg(not(target_os = "android"))]
pub mod ytdl;

use crate::AppState;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

/// Build the complete API router with all route groups.
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .nest("/api/libraries", libraries::router())
        .nest("/api/media", media::router())
        .nest("/api/media-lists", media_lists::router())
        .nest("/api/downloads", downloads::router())
        .nest("/api/database", database::router())
        .nest("/api/youtube", youtube::router())
        .nest("/api/youtube-search", youtube_search::router());

    #[cfg(not(target_os = "android"))]
    let router = router.nest("/api/ytdl", ytdl::router());

    #[cfg(feature = "embed-frontend")]
    let router = router.fallback(frontend::serve_frontend);

    router.with_state(state).layer(cors)
}
