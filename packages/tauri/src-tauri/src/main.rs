#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.get(1).is_some_and(|a| a == "server") {
        run_server();
    } else {
        mhaoltube::run();
    }
}

#[tokio::main]
async fn run_server() {
    use mhaoltube::{api, load_env_app, AppState};
    use std::path::PathBuf;
    use tokio::net::TcpListener;

    load_env_app();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mhaoltube=info".into()),
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
        .unwrap_or_else(|| mhaoltube::default_data_dir().join("mhaoltube.db"));

    let state = AppState::new(Some(db_path.as_path())).expect("Failed to initialize database");

    state.seed_default_library();
    state.initialize_modules();

    let app = api::build_router(state);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Backend server listening on {}", addr);

    axum::serve(listener, app).await.expect("Server error");
}
