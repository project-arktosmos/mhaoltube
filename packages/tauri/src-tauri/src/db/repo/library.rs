use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRow {
    pub id: String,
    pub name: String,
    pub path: String,
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
            "SELECT id, name, path, date_added, created_at, updated_at FROM libraries WHERE id = ?1",
            params![id],
            |row| {
                Ok(LibraryRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    date_added: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .ok()
    }

    pub fn insert(&self, id: &str, name: &str, path: &str, date_added: i64) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO libraries (id, name, path, date_added) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, path, date_added],
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

        repo.insert("default", "Library", "/tmp/mhaoltube", 1000);
        let lib = repo.get("default").unwrap();
        assert_eq!(lib.name, "Library");
        assert_eq!(lib.path, "/tmp/mhaoltube");
    }
}
