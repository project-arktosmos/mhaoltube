use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRow {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_types: String,
    pub date_added: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct LibraryRepo {
    db: DbPool,
}

impl LibraryRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<LibraryRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, path, media_types, date_added, created_at, updated_at FROM libraries WHERE id = ?1",
            params![id],
            |row| {
                Ok(LibraryRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    media_types: row.get(3)?,
                    date_added: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .ok()
    }

    pub fn insert(&self, id: &str, name: &str, path: &str, media_types: &str, date_added: i64) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO libraries (id, name, path, media_types, date_added) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, path, media_types, date_added],
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_library_insert_and_get() {
        let db = open_test_database();
        let repo = LibraryRepo::new(db);

        repo.insert("default", "Library", "/tmp/mhaoltube", "[\"video\",\"image\",\"audio\",\"other\"]", 1000);
        let lib = repo.get("default").unwrap();
        assert_eq!(lib.name, "Library");
        assert_eq!(lib.path, "/tmp/mhaoltube");
        assert_eq!(lib.media_types, "[\"video\",\"image\",\"audio\",\"other\"]");
    }
}
