use std::path::PathBuf;

use error_generator::error;
use rusqlite::{Connection, Statement};

use crate::new_tube_service::yt_dlp::{Item, Items};

type Result<T> = std::result::Result<T, DBError>;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let connection = Connection::open(Self::get_path())?;

        connection.execute("\
        CREATE TABLE IF NOT EXISTS PlaylistItems (
            playlist_id TEXT PRIMARY KEY,
            video_id TEXT NOT NULL,
            title TEXT NOT NULL,
            duration REAL NOT NULL,
            uploader TEXT NOT NULL
        );", [])?;

        Ok(Database { connection })
    }

    fn get_path() -> PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("new_tube.db");
        path
    }

    pub fn get_items(&self) -> Result<Items> {
        let mut statement = self.create_select_all_statement()?;

        let result = statement.query_map([], |row| {
            Item::try_from(row)
        })?;
        Ok(result.map(|r| r.unwrap()).collect())
    }

    pub fn get_playlist_ids(&self) -> Result<Vec<String>> {
        Ok(self.get_items()?.into_iter().map(|item| item.playlist_id).collect())
    }

    fn create_select_all_statement(&self) -> Result<Statement> {
        Ok(self.connection.prepare("\
            SELECT * FROM PlaylistItems;
        ")?)
    }

    pub fn add_item(&self, item: &Item) -> Result<()> {
        self.connection.execute("\
            INSERT OR REPLACE INTO PlaylistItems (playlist_id, video_id, title, duration, uploader)
            VALUES (?1, ?2, ?3, ?4, ?5);
        ", &[&item.playlist_id, &item.video_id, &item.title, &format!("{}", item.duration.unwrap_or_default()), &item.uploader])?;

        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.connection.execute("DELETE FROM PlaylistItems WHERE playlist_id = ?1", &[id])?;
        Ok(())
    }
}

#[error(message = "Error while connecting to the database or while executing queries: {self.0}", impl_from)]
pub struct DBError(rusqlite::Error);