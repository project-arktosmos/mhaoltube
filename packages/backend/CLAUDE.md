# Package: backend

**Location:** `packages/backend/`
**Framework:** Rust — Axum 0.8 + Tokio + rusqlite (SQLite)
**Binary:** `mhaoltube-server` (default port 1530)

## Source Structure

```
src/
├── server.rs             # Binary entry point — starts Axum server
├── lib.rs                # AppState definition, module declarations
├── api/                  # Route handlers (one module per feature)
│   ├── mod.rs            # build_router() — registers all routes
│   ├── database.rs
│   ├── downloads.rs
│   ├── libraries.rs
│   ├── media.rs
│   ├── media_lists.rs
│   ├── player.rs
│   ├── youtube.rs
│   ├── youtube_search.rs
│   └── ytdl.rs           # cfg(not(target_os = "android"))
├── db/                   # Database layer (rusqlite repos)
└── modules/              # Plugin modules (youtube_meta, ytdl)
```

## AppState

All API handlers receive `AppState` which contains:
- Database repositories (settings, metadata, libraries, library_items, etc.)
- `ModuleRegistry` for plugin modules
- `DownloadManager` (yt-dlp, desktop only)

## Adding a New API Module

1. Create `src/api/{feature}.rs` with a `pub fn router() -> Router<AppState>` function
2. Add `pub mod {feature};` to `src/api/mod.rs`
3. Register in `build_router()`: `.nest("/api/{feature}", {feature}::router())`
4. If new database access is needed, add a repo to `AppState`

## Sub-crate Dependencies (desktop only)

These are conditionally compiled with `#[cfg(not(target_os = "android"))]`:
- `mhaoltube-yt-dlp` — YouTube downloading (`packages/rust-yt-dlp/`)

## Running

```bash
# From repo root
pnpm dev:backend          # PORT=1530 cargo run -p mhaoltube-backend --bin mhaoltube-server
pnpm build:backend        # cargo build -p mhaoltube-backend --release

# Tests
cargo test -p mhaoltube-backend
cargo check -p mhaoltube-backend
```

## Environment Variables

- `PORT` — Server port (default: 1530)
- `HOST` — Bind address (default: 0.0.0.0)
- `DB_PATH` — SQLite database path (optional, in-memory if unset)
