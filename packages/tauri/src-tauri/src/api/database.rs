use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tables", get(list_tables))
        .route("/tables/{name}", get(get_table))
        .route("/reset", post(reset_database))
}

#[derive(Serialize)]
struct ColumnInfo {
    name: String,
    #[serde(rename = "type")]
    col_type: String,
}

#[derive(Serialize)]
struct TableInfo {
    name: String,
    columns: Vec<ColumnInfo>,
    #[serde(rename = "rowCount")]
    row_count: i64,
}

async fn list_tables(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock();
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .unwrap();
    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let mut result: Vec<TableInfo> = Vec::new();
    for name in table_names {
        let columns = get_table_columns(&conn, &name);
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM \"{}\"", name), [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        result.push(TableInfo {
            name,
            columns,
            row_count: count,
        });
    }

    Json(result)
}

#[derive(Deserialize)]
struct TableQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct TableDetailResponse {
    table: String,
    columns: Vec<ColumnInfo>,
    rows: Vec<serde_json::Value>,
    pagination: Pagination,
}

#[derive(Serialize)]
struct Pagination {
    page: i64,
    limit: i64,
    total: i64,
    #[serde(rename = "totalPages")]
    total_pages: i64,
}

async fn get_table(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<TableQuery>,
) -> impl IntoResponse {
    let conn = state.db.lock();

    // Check table exists
    let exists: bool = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?1 AND name NOT LIKE 'sqlite_%'")
        .unwrap()
        .exists(params![name])
        .unwrap_or(false);

    if !exists {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Table \"{}\" not found", name) })),
        )
            .into_response();
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM \"{}\"", name), [], |r| {
            r.get(0)
        })
        .unwrap_or(0);

    let columns = get_table_columns(&conn, &name);
    let col_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

    let sql = format!("SELECT * FROM \"{}\" LIMIT ?1 OFFSET ?2", name);
    let mut stmt = conn.prepare(&sql).unwrap();
    let rows: Vec<serde_json::Value> = stmt
        .query_map(params![limit, offset], |row| {
            let mut map = serde_json::Map::new();
            for (i, col_name) in col_names.iter().enumerate() {
                let val: rusqlite::types::Value = row.get(i)?;
                let json_val = match val {
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(n) => serde_json::Value::from(n),
                    rusqlite::types::Value::Real(f) => serde_json::json!(f),
                    rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                    rusqlite::types::Value::Blob(b) => {
                        serde_json::Value::String(format!("<blob:{} bytes>", b.len()))
                    }
                };
                map.insert(col_name.clone(), json_val);
            }
            Ok(serde_json::Value::Object(map))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let total_pages = if limit > 0 {
        (total + limit - 1) / limit
    } else {
        0
    };

    Json(serde_json::json!(TableDetailResponse {
        table: name,
        columns,
        rows,
        pagination: Pagination {
            page,
            limit,
            total,
            total_pages,
        },
    }))
    .into_response()
}

async fn reset_database(State(state): State<AppState>) -> impl IntoResponse {
    // Step 1: Wipe media files (audio/ and video/ subdirs) without touching the
    // live SQLite db file. Deleting the db file while the connection is open
    // causes the WAL salt to mismatch on the next write, silently discarding
    // DDL changes and leaving the old data in place.
    let library_dir = crate::default_data_dir();
    for subdir in &["audio", "video"] {
        let dir = library_dir.join(subdir);
        if dir.exists() {
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    // Step 2: Ensure the library directory and media subdirs exist.
    std::fs::create_dir_all(&library_dir).ok();

    // Step 3 & 4: Drop everything in the db and regenerate schema + seed data.
    {
        let conn = state.db.lock();

        conn.execute_batch("PRAGMA foreign_keys = OFF").unwrap();

        let triggers: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        for trigger in &triggers {
            conn.execute_batch(&format!("DROP TRIGGER IF EXISTS \"{}\"", trigger))
                .unwrap();
        }

        let tables: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        for table in &tables {
            conn.execute_batch(&format!("DROP TABLE IF EXISTS \"{}\"", table))
                .unwrap();
        }

        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();

        crate::db::schema::initialize_schema(&conn).unwrap();
        crate::db::schema::initialize_module_schemas(&conn).unwrap();
    } // mutex released here

    // Step 4 (continued): seed the default library record.
    state.seed_default_library();

    // Always recreate media subdirs — seed_default_library only creates them
    // when inserting a new library row, so an explicit pass here ensures they
    // exist regardless of guard conditions.
    for subdir in &["audio", "video"] {
        let dir = library_dir.join(subdir);
        std::fs::create_dir_all(dir.join(".cache")).ok();
    }

    // Re-seed module settings (ModuleRegistry.initialize() only runs once at
    // startup, so we re-apply defaults manually after a reset).
    state.module_registry.read().seed_settings(&state);

    // Sync the library: scan audio/ and video/ on disk and reconcile with
    // the youtube_content table so the DB reflects actual filesystem state.
    sync_library(&state);

    tracing::info!("Database reset complete");

    let conn = state.db.lock();
    let new_tables: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    Json(serde_json::json!({ "ok": true, "tables": new_tables }))
}

/// Scan the library's audio/ and video/ subdirectories and remove any
/// youtube_content rows whose files no longer exist on disk.
fn sync_library(state: &crate::AppState) {
    let library_dir = crate::default_data_dir();
    let rows = state.youtube_content.get_all();

    for row in rows {
        let video_ok = row
            .video_path
            .as_deref()
            .map(|p| std::path::Path::new(p).exists())
            .unwrap_or(true);
        let audio_ok = row
            .audio_path
            .as_deref()
            .map(|p| std::path::Path::new(p).exists())
            .unwrap_or(true);

        if !video_ok && !audio_ok {
            state.youtube_content.delete(&row.youtube_id);
        } else {
            if !video_ok {
                state.youtube_content.clear_video_path(&row.youtube_id);
            }
            if !audio_ok {
                state.youtube_content.clear_audio_path(&row.youtube_id);
            }
        }
    }

    tracing::info!(
        "Library sync complete (scanned {})",
        library_dir.display()
    );
}

fn get_table_columns(conn: &rusqlite::Connection, table: &str) -> Vec<ColumnInfo> {
    let sql = format!("PRAGMA table_info('{}')", table);
    let mut stmt = conn.prepare(&sql).unwrap();
    stmt.query_map([], |row| {
        Ok(ColumnInfo {
            name: row.get(1)?,
            col_type: row.get(2)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}
