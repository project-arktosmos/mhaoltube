use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeChannelRow {
    pub id: String,
    pub handle: String,
    pub name: String,
    pub url: String,
    pub subscriber_text: Option<String>,
    pub image_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeChannelUpdate {
    pub name: Option<String>,
    pub subscriber_text: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Clone)]
pub struct YouTubeChannelRepo {
    db: DbPool,
}

const SELECT_COLS: &str =
    "id, handle, name, url, subscriber_text, image_url, created_at, updated_at";

fn row_to_channel(row: &rusqlite::Row) -> rusqlite::Result<YouTubeChannelRow> {
    Ok(YouTubeChannelRow {
        id: row.get(0)?,
        handle: row.get(1)?,
        name: row.get(2)?,
        url: row.get(3)?,
        subscriber_text: row.get(4)?,
        image_url: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

impl YouTubeChannelRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<YouTubeChannelRow> {
        let conn = self.db.lock();
        conn.query_row(
            &format!(
                "SELECT {} FROM youtube_channels WHERE id = ?1",
                SELECT_COLS
            ),
            params![id],
            row_to_channel,
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<YouTubeChannelRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM youtube_channels ORDER BY name ASC",
                SELECT_COLS
            ))
            .unwrap();
        stmt.query_map([], row_to_channel)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn insert(&self, channel: &YouTubeChannelRow) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "INSERT OR IGNORE INTO youtube_channels (id, handle, name, url, subscriber_text, image_url) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                channel.id,
                channel.handle,
                channel.name,
                channel.url,
                channel.subscriber_text,
                channel.image_url,
            ],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
    }

    pub fn update(&self, id: &str, update: &YouTubeChannelUpdate) -> bool {
        let conn = self.db.lock();
        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref name) = update.name {
            sets.push("name = ?");
            values.push(Box::new(name.clone()));
        }
        if let Some(ref subscriber_text) = update.subscriber_text {
            sets.push("subscriber_text = ?");
            values.push(Box::new(subscriber_text.clone()));
        }
        if let Some(ref image_url) = update.image_url {
            sets.push("image_url = ?");
            values.push(Box::new(image_url.clone()));
        }

        if sets.is_empty() {
            return false;
        }

        values.push(Box::new(id.to_string()));
        let sql = format!(
            "UPDATE youtube_channels SET {} WHERE id = ?",
            sets.join(", ")
        );
        let params: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql, params.as_slice())
            .map(|n| n > 0)
            .unwrap_or(false)
    }

    pub fn delete(&self, id: &str) -> bool {
        let conn = self.db.lock();
        conn.execute("DELETE FROM youtube_channels WHERE id = ?1", params![id])
            .map(|n| n > 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_channels_seeded() {
        let db = open_test_database();
        let repo = YouTubeChannelRepo::new(db);

        let all = repo.get_all();
        assert_eq!(all.len(), 15);

        let channel = repo.get("yt-cinemasins").unwrap();
        assert_eq!(channel.handle, "CinemaSins");
        assert_eq!(channel.name, "CinemaSins");
        assert!(channel.subscriber_text.is_none());
        assert!(channel.image_url.is_none());
    }

    #[test]
    fn test_insert_and_update() {
        let db = open_test_database();
        let repo = YouTubeChannelRepo::new(db);

        let channel = YouTubeChannelRow {
            id: "yt-test".to_string(),
            handle: "TestChannel".to_string(),
            name: "Test Channel".to_string(),
            url: "https://www.youtube.com/@TestChannel".to_string(),
            subscriber_text: Some("1M subscribers".to_string()),
            image_url: Some("https://example.com/avatar.jpg".to_string()),
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert!(repo.insert(&channel));

        let fetched = repo.get("yt-test").unwrap();
        assert_eq!(fetched.subscriber_text.as_deref(), Some("1M subscribers"));
        assert_eq!(
            fetched.image_url.as_deref(),
            Some("https://example.com/avatar.jpg")
        );

        let update = YouTubeChannelUpdate {
            name: None,
            subscriber_text: Some("2M subscribers".to_string()),
            image_url: None,
        };
        assert!(repo.update("yt-test", &update));

        let updated = repo.get("yt-test").unwrap();
        assert_eq!(updated.subscriber_text.as_deref(), Some("2M subscribers"));
    }

    #[test]
    fn test_delete() {
        let db = open_test_database();
        let repo = YouTubeChannelRepo::new(db);

        assert!(repo.get("yt-cinemasins").is_some());
        assert!(repo.delete("yt-cinemasins"));
        assert!(repo.get("yt-cinemasins").is_none());
    }
}
