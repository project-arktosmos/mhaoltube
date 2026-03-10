pub mod api;
pub mod db;
pub mod modules;

use db::repo::*;
use db::DbPool;
use mhaoltube_yt_dlp::DownloadManager;
use modules::ModuleRegistry;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Return the default mhaoltube data directory: `<Documents>/mhaoltube`.
/// Creates the directory if it does not exist.
pub fn default_data_dir() -> PathBuf {
    let doc_dir = dirs::document_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join("Documents")
    });
    let data_dir = doc_dir.join("mhaoltube");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir
}

/// Load .env.app from the workspace root into process environment variables.
/// Only sets variables that are not already present in the environment.
pub fn load_env_app() {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let env_path = loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            break dir.join(".env.app");
        }
        if !dir.pop() {
            break PathBuf::from(".env.app");
        }
    };

    let content = match std::fs::read_to_string(&env_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim();
            let value = trimmed[eq_idx + 1..].trim();
            if !key.is_empty() && std::env::var(key).is_err() {
                std::env::set_var(key, value);
            }
        }
    }
}

/// Shared application state available to all API handlers and modules.
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub settings: SettingsRepo,
    pub metadata: MetadataRepo,
    pub libraries: LibraryRepo,
    pub youtube_content: YouTubeContentRepo,
    pub youtube_downloads: YouTubeDownloadRepo,
    pub youtube_channels: YouTubeChannelRepo,
    pub module_registry: Arc<RwLock<ModuleRegistry>>,
    pub data_dir: PathBuf,
    pub ytdl_manager: Arc<DownloadManager>,
}

impl AppState {
    /// Create a new AppState with a database at the given path (or in-memory if None).
    pub fn new(db_path: Option<&Path>, data_dir: PathBuf) -> Result<Self, rusqlite::Error> {
        let db = db::open_database(db_path)?;

        Ok(Self {
            settings: SettingsRepo::new(Arc::clone(&db)),
            metadata: MetadataRepo::new(Arc::clone(&db)),
            libraries: LibraryRepo::new(Arc::clone(&db)),
            youtube_content: YouTubeContentRepo::new(Arc::clone(&db)),
            youtube_downloads: YouTubeDownloadRepo::new(Arc::clone(&db)),
            youtube_channels: YouTubeChannelRepo::new(Arc::clone(&db)),
            module_registry: Arc::new(RwLock::new(ModuleRegistry::new())),
            data_dir,
            ytdl_manager: {
                let config = mhaoltube_yt_dlp::YtDownloadConfig::from_env();
                Arc::new(DownloadManager::new(config))
            },
            db,
        })
    }

    /// Register and initialize all built-in modules.
    pub fn initialize_modules(&self) {
        use modules::youtube_meta::YoutubeMetaModule;
        use modules::ytdl::YtdlModule;

        let mut registry = self.module_registry.write();

        registry.register(Box::new(YoutubeMetaModule));
        registry.register(Box::new(YtdlModule {
            manager: Arc::clone(&self.ytdl_manager),
        }));

        registry.initialize(self);
    }

    /// Backfill `youtube_content` from completed `youtube_downloads` rows.
    ///
    /// `youtube_content.upsert()` normally runs inside the SSE stream handler, which only
    /// executes while a client is actively subscribed. Any download that completed while no
    /// browser tab was connected never got written to `youtube_content`. This method repairs
    /// that gap at startup.
    pub fn sync_downloads_to_content(&self) {
        let completed = self.youtube_downloads.get_by_state("completed");
        for row in completed {
            let Some(ref output_path) = row.output_path else {
                continue;
            };
            if !std::path::Path::new(output_path).exists() {
                continue;
            }
            // mode is stored as a JSON-serialised string, e.g. `"audio"` (with quotes)
            let mode = row.mode.trim_matches('"');
            let existing = self.youtube_content.get(&row.video_id);
            let (video_path, audio_path): (Option<&str>, Option<&str>) = match mode {
                "audio" => {
                    if existing.as_ref().map_or(false, |e| e.audio_path.is_some()) {
                        continue;
                    }
                    (None, Some(output_path.as_str()))
                }
                "video" => {
                    if existing.as_ref().map_or(false, |e| e.video_path.is_some()) {
                        continue;
                    }
                    (Some(output_path.as_str()), None)
                }
                _ => continue, // "both": can't infer audio vs video from a single output_path
            };
            self.youtube_content.upsert(
                &row.video_id,
                &row.title,
                row.thumbnail_url.as_deref(),
                row.duration_seconds,
                None,
                None,
                video_path,
                audio_path,
            );
        }
        tracing::info!("[library] synced completed downloads to youtube_content");
    }

    /// Spawn a background task that writes completed downloads into `youtube_content`.
    ///
    /// Unlike the SSE handler, this subscriber runs for the lifetime of the process and
    /// is not tied to any browser client being connected. Every completed download will
    /// be recorded in `youtube_content` regardless of SSE state.
    pub fn start_content_sync_task(&self) {
        use mhaoltube_yt_dlp::{manager::SseEvent, DownloadState};
        use tokio::sync::broadcast::error::RecvError;

        let mut rx = self.ytdl_manager.subscribe_events();
        let youtube_content = self.youtube_content.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(SseEvent::Progress(progress)) => {
                        if progress.state == DownloadState::Completed {
                            youtube_content.upsert(
                                &progress.video_id,
                                &progress.title,
                                progress.thumbnail_url.as_deref(),
                                progress.duration_seconds.map(|d| d as i64),
                                progress.channel_name.as_deref(),
                                None,
                                progress.video_output_path.as_deref(),
                                progress.audio_output_path.as_deref(),
                            );
                            tracing::info!(
                                "[library] recorded completed download {} ('{}') to youtube_content",
                                progress.video_id,
                                progress.title
                            );
                        }
                    }
                    Ok(_) => {}
                    Err(RecvError::Lagged(n)) => {
                        tracing::warn!("[library] content sync task lagged by {} events", n);
                    }
                    Err(RecvError::Closed) => {
                        tracing::info!("[library] content sync task: channel closed");
                        break;
                    }
                }
            }
        });
    }

    /// The fixed ID for the single default library.
    pub const DEFAULT_LIBRARY_ID: &'static str = "default";

    /// Ensure the single default library exists, located in the mhaoltube data directory.
    pub fn seed_default_library(&self) {
        if self.libraries.get(Self::DEFAULT_LIBRARY_ID).is_some() {
            return;
        }
        let library_path = &self.data_dir;
        let library_path_str = library_path.to_string_lossy();
        self.libraries.insert(
            Self::DEFAULT_LIBRARY_ID,
            "Library",
            &library_path_str,
            chrono::Utc::now().timestamp_millis(),
        );
        self.metadata
            .set_string("youtube.libraryId", Self::DEFAULT_LIBRARY_ID);
        for subdir in &["audio", "video"] {
            let dir = library_path.join(subdir);
            std::fs::create_dir_all(dir.join(".cache")).ok();
        }
        tracing::info!("Created default library at {}", library_path_str);
    }
}

// --- Tauri commands ---

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// --- Tauri desktop entry point ---

const SERVER_PORT: u16 = 1530;
const SERVER_HOST: &str = "127.0.0.1";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_env_app();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![open_path])
        .setup(|app| {
            // Start the Rust backend server unless one is already running on the port
            let addr = format!("{}:{}", SERVER_HOST, SERVER_PORT);
            let already_running = std::net::TcpStream::connect(&addr).is_ok();

            if already_running {
                eprintln!("Backend already running on {}, skipping embedded server", addr);
            } else {
                // On Android, dirs::document_dir() is unavailable; use Tauri's
                // app data directory so the DB lands in private app storage.
                #[cfg(target_os = "android")]
                let data_dir = {
                    use tauri::Manager;
                    app.path()
                        .data_dir()
                        .expect("failed to resolve app data dir")
                        .join("mhaoltube")
                };
                #[cfg(not(target_os = "android"))]
                let data_dir = default_data_dir();
                let _ = app; // suppress unused warning on desktop

                std::fs::create_dir_all(&data_dir).ok();
                let db_path = data_dir.join("mhaoltube.db");
                tauri::async_runtime::spawn(async move {
                    let state = AppState::new(Some(db_path.as_path()), data_dir)
                        .expect("failed to initialize backend");
                    state.seed_default_library();
                    state.initialize_modules();
                    state.sync_downloads_to_content();
                    state.start_content_sync_task();
                    let router = api::build_router(state);
                    let addr = format!("{}:{}", SERVER_HOST, SERVER_PORT);
                    let listener = tokio::net::TcpListener::bind(&addr)
                        .await
                        .expect("failed to bind backend server");
                    axum::serve(listener, router)
                        .await
                        .expect("backend server error");
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
