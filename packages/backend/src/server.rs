use mhaoltube_backend::{api, load_env_app, AppState};
use std::path::PathBuf;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    load_env_app();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mhaoltube_backend=info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1530);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let db_path = std::env::var("DB_PATH")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let candidate = PathBuf::from(manifest_dir)
                .parent()
                .and_then(|p| p.parent())
                .map(|root| root.join("packages/database/mhaoltube.db"));
            match candidate {
                Some(p) if p.parent().map_or(false, |d| d.exists()) => p,
                _ => PathBuf::from("mhaoltube.db"),
            }
        });

    let state = AppState::new(Some(db_path.as_path()))
        .expect("Failed to initialize database");

    state.seed_default_library();
    state.initialize_modules();

    let app = api::build_router(state);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Backend server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
