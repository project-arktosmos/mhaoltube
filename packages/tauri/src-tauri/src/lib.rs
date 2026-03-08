pub mod api;
pub mod db;
pub mod modules;

use db::repo::*;
use db::DbPool;
#[cfg(not(target_os = "android"))]
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
    pub library_items: LibraryItemRepo,
    pub library_item_links: LibraryItemLinkRepo,
    pub media_types: MediaTypeRepo,
    pub categories: CategoryRepo,
    pub link_sources: LinkSourceRepo,
    pub youtube_downloads: YouTubeDownloadRepo,
    pub media_lists: MediaListRepo,
    pub media_list_items: MediaListItemRepo,
    pub media_list_links: MediaListLinkRepo,
    pub module_registry: Arc<RwLock<ModuleRegistry>>,
    #[cfg(not(target_os = "android"))]
    pub ytdl_manager: Arc<DownloadManager>,
}

impl AppState {
    /// Create a new AppState with a database at the given path (or in-memory if None).
    pub fn new(db_path: Option<&Path>) -> Result<Self, rusqlite::Error> {
        let db = db::open_database(db_path)?;

        Ok(Self {
            settings: SettingsRepo::new(Arc::clone(&db)),
            metadata: MetadataRepo::new(Arc::clone(&db)),
            libraries: LibraryRepo::new(Arc::clone(&db)),
            library_items: LibraryItemRepo::new(Arc::clone(&db)),
            library_item_links: LibraryItemLinkRepo::new(Arc::clone(&db)),
            media_types: MediaTypeRepo::new(Arc::clone(&db)),
            categories: CategoryRepo::new(Arc::clone(&db)),
            link_sources: LinkSourceRepo::new(Arc::clone(&db)),
            youtube_downloads: YouTubeDownloadRepo::new(Arc::clone(&db)),
            media_lists: MediaListRepo::new(Arc::clone(&db)),
            media_list_items: MediaListItemRepo::new(Arc::clone(&db)),
            media_list_links: MediaListLinkRepo::new(Arc::clone(&db)),
            module_registry: Arc::new(RwLock::new(ModuleRegistry::new())),
            #[cfg(not(target_os = "android"))]
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
        #[cfg(not(target_os = "android"))]
        use modules::ytdl::YtdlModule;

        let mut registry = self.module_registry.write();

        registry.register(Box::new(YoutubeMetaModule));

        #[cfg(not(target_os = "android"))]
        {
            registry.register(Box::new(YtdlModule {
                manager: Arc::clone(&self.ytdl_manager),
            }));
        }

        registry.initialize(self);
    }

    /// Seed a default library if no libraries exist, located in the mhaoltube data directory.
    pub fn seed_default_library(&self) {
        if self.libraries.get_all().is_empty() {
            let library_path = default_data_dir();
            let library_path_str = library_path.to_string_lossy();
            self.libraries.insert(
                &uuid::Uuid::new_v4().to_string(),
                "Default",
                &library_path_str,
                "[\"video\",\"image\",\"audio\",\"other\"]",
                chrono::Utc::now().timestamp_millis(),
            );
            tracing::info!("Created default library at {}", library_path_str);
        }
    }
}

// --- Tauri desktop entry point ---

const SERVER_PORT: u16 = 1530;
const SERVER_HOST: &str = "127.0.0.1";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_env_app();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // Start the Rust backend server unless one is already running on the port
            let addr = format!("{}:{}", SERVER_HOST, SERVER_PORT);
            let already_running = std::net::TcpStream::connect(&addr).is_ok();

            if already_running {
                eprintln!("Backend already running on {}, skipping embedded server", addr);
            } else {
                let db_path = default_data_dir().join("mhaoltube.db");
                tauri::async_runtime::spawn(async move {
                    let state = AppState::new(Some(db_path.as_path()))
                        .expect("failed to initialize backend");
                    state.seed_default_library();
                    state.initialize_modules();
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
