use rusqlite::Connection;

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS settings_updated_at
    AFTER UPDATE ON settings
    FOR EACH ROW
BEGIN
    UPDATE settings SET updated_at = datetime('now') WHERE key = OLD.key;
END;

CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'string' CHECK (type IN ('string', 'number', 'boolean', 'json')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS metadata_updated_at
    AFTER UPDATE ON metadata
    FOR EACH ROW
BEGIN
    UPDATE metadata SET updated_at = datetime('now') WHERE key = OLD.key;
END;

CREATE TABLE IF NOT EXISTS libraries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    date_added INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS libraries_updated_at
    AFTER UPDATE ON libraries
    FOR EACH ROW
BEGIN
    UPDATE libraries SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS youtube_content (
    youtube_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    thumbnail_url TEXT,
    duration_seconds INTEGER,
    channel_name TEXT,
    channel_id TEXT,
    video_path TEXT,
    audio_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS youtube_content_updated_at
    AFTER UPDATE ON youtube_content
    FOR EACH ROW
BEGIN
    UPDATE youtube_content SET updated_at = datetime('now') WHERE youtube_id = OLD.youtube_id;
END;

CREATE TABLE IF NOT EXISTS youtube_downloads (
    download_id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    video_id TEXT NOT NULL,
    title TEXT NOT NULL,
    state TEXT NOT NULL,
    progress REAL NOT NULL,
    downloaded_bytes INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL,
    output_path TEXT,
    error TEXT,
    mode TEXT NOT NULL,
    quality TEXT NOT NULL,
    format TEXT NOT NULL,
    video_quality TEXT,
    video_format TEXT,
    thumbnail_url TEXT,
    duration_seconds INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS youtube_channels (
    id TEXT PRIMARY KEY,
    handle TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    subscriber_text TEXT,
    image_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS youtube_channels_updated_at
    AFTER UPDATE ON youtube_channels
    FOR EACH ROW
BEGIN
    UPDATE youtube_channels SET updated_at = datetime('now') WHERE id = OLD.id;
END;
";

const SEED_SQL: &str = "
INSERT OR REPLACE INTO metadata (key, value, type) VALUES ('db_version', '23', 'number');
INSERT OR IGNORE INTO metadata (key, value, type) VALUES ('created_at', datetime('now'), 'string');

INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-teamfourstar', 'TeamFourStar', 'TeamFourStar', 'https://www.youtube.com/@TeamFourStar');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-cinemasins', 'CinemaSins', 'CinemaSins', 'https://www.youtube.com/@CinemaSins');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-lsmark', 'LSMark', 'LS Mark', 'https://www.youtube.com/@LSMark');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-coffeeandcults', 'coffeeandcults', 'Coffee and Cults', 'https://www.youtube.com/@coffeeandcults');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-uncleherman', 'uncleherman', 'uncle herman', 'https://www.youtube.com/@uncleherman');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-chrisjamestv', 'ChrisJamesTV', 'Chris James', 'https://www.youtube.com/@ChrisJamesTV');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-sloanstowe', 'SloanStowe', 'Sloan Stowe', 'https://www.youtube.com/@SloanStowe');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-pitchmeetings', 'PitchMeetings', 'Pitch Meeting', 'https://www.youtube.com/@PitchMeetings');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-vengamonjas', 'vengamonjas', 'Venga Monjas', 'https://www.youtube.com/@vengamonjas');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-kallmekris', 'kallmekris', 'Kallmekris', 'https://www.youtube.com/@kallmekris');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-nitpik', 'NitPik-p3r', 'NitPik', 'https://www.youtube.com/@NitPik-p3r');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-myfriendscallmepat', 'myfriendscallmepat', 'My Friends Call Me Pat', 'https://www.youtube.com/@myfriendscallmepat');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-deepdive', 'DeepDiveProductions', 'Deep Dive', 'https://www.youtube.com/@DeepDiveProductions');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-tvsins', 'TVSins', 'TVSins', 'https://www.youtube.com/@TVSins');
INSERT OR IGNORE INTO youtube_channels (id, handle, name, url) VALUES ('yt-paperwill', 'PaperWill', 'Paper Will', 'https://www.youtube.com/@PaperWill');
";

pub const YOUTUBE_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS youtube_videos (
    video_id TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

fn has_table(conn: &Connection, name: &str) -> bool {
    conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")
        .unwrap()
        .exists(rusqlite::params![name])
        .unwrap_or(false)
}

fn run_migrations(conn: &Connection) {
    // Migration: migrate completed youtube_downloads into youtube_content
    if has_table(conn, "youtube_downloads") && has_table(conn, "youtube_content") {
        let _ = conn.execute_batch(
            "INSERT OR IGNORE INTO youtube_content (youtube_id, title, thumbnail_url, duration_seconds, video_path, audio_path)
             SELECT video_id, title, thumbnail_url, duration_seconds,
                    CASE WHEN mode = 'video' THEN output_path ELSE NULL END,
                    CASE WHEN mode = 'audio' THEN output_path ELSE NULL END
             FROM youtube_downloads WHERE state = 'completed' AND output_path IS NOT NULL;",
        );
    }

    // Migration: drop legacy tables from older databases
    let _ = conn.execute_batch(
        "DROP TABLE IF EXISTS media_list_items;
         DROP TABLE IF EXISTS media_list_links;
         DROP TABLE IF EXISTS media_lists;
         DROP TABLE IF EXISTS library_item_links;
         DROP TABLE IF EXISTS library_items;
         DROP TABLE IF EXISTS link_sources;
         DROP TABLE IF EXISTS categories;
         DROP TABLE IF EXISTS media_types;
         DROP TABLE IF EXISTS torrent_downloads;
         DROP TABLE IF EXISTS image_tags;
         DROP TABLE IF EXISTS signaling_servers;",
    );

    // Migration: drop media_types column from libraries if it exists
    // SQLite doesn't support DROP COLUMN before 3.35.0, so we recreate the table
    if has_table(conn, "libraries") {
        let has_media_types: bool = {
            let sql = "PRAGMA table_info(libraries)";
            let mut stmt = conn.prepare(sql).unwrap();
            let columns: Vec<String> = stmt
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            columns.iter().any(|c| c == "media_types")
        };
        if has_media_types {
            let _ = conn.execute_batch(
                "CREATE TABLE libraries_new (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    date_added INTEGER NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                INSERT INTO libraries_new (id, name, path, date_added, created_at, updated_at)
                SELECT id, name, path, date_added, created_at, updated_at FROM libraries;
                DROP TABLE libraries;
                ALTER TABLE libraries_new RENAME TO libraries;
                CREATE TRIGGER IF NOT EXISTS libraries_updated_at
                    AFTER UPDATE ON libraries FOR EACH ROW
                BEGIN UPDATE libraries SET updated_at = datetime('now') WHERE id = OLD.id; END;",
            );
        }
    }

    // Migration: consolidate all libraries into a single "default" library
    if has_table(conn, "libraries") {
        let has_default: bool = conn
            .prepare("SELECT id FROM libraries WHERE id = 'default'")
            .and_then(|mut s| s.exists([]))
            .unwrap_or(false);
        if !has_default {
            let first_lib_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM libraries ORDER BY date_added ASC LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok();
            if let Some(old_id) = first_lib_id {
                let _ = conn.execute_batch(&format!(
                    "UPDATE libraries SET id = 'default', name = 'Library' WHERE id = '{}';
                     DELETE FROM libraries WHERE id != 'default';",
                    old_id
                ));
            }
        } else {
            let _ = conn.execute_batch("DELETE FROM libraries WHERE id != 'default';");
        }
    }
}

/// Initialize the database schema, run migrations, and seed data.
pub fn initialize_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(SCHEMA_SQL)?;
    run_migrations(conn);
    conn.execute_batch(SEED_SQL)?;
    Ok(())
}

/// Apply module schemas (addon tables).
pub fn initialize_module_schemas(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(YOUTUBE_SCHEMA_SQL)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_schema() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        // Verify core tables exist
        assert!(has_table(&conn, "settings"));
        assert!(has_table(&conn, "metadata"));
        assert!(has_table(&conn, "libraries"));
        assert!(has_table(&conn, "youtube_content"));
        assert!(has_table(&conn, "youtube_downloads"));
        assert!(has_table(&conn, "youtube_channels"));

        // Verify removed tables don't exist
        assert!(!has_table(&conn, "media_types"));
        assert!(!has_table(&conn, "categories"));
        assert!(!has_table(&conn, "library_items"));
        assert!(!has_table(&conn, "library_item_links"));
        assert!(!has_table(&conn, "link_sources"));
        assert!(!has_table(&conn, "media_lists"));
        assert!(!has_table(&conn, "media_list_items"));
        assert!(!has_table(&conn, "media_list_links"));
        assert!(!has_table(&conn, "torrent_downloads"));
        assert!(!has_table(&conn, "image_tags"));
        assert!(!has_table(&conn, "signaling_servers"));

        // Verify seed data
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM youtube_channels", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 15);
    }

    #[test]
    fn test_initialize_module_schemas() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        initialize_module_schemas(&conn).unwrap();

        assert!(has_table(&conn, "youtube_videos"));
    }
}
