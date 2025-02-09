use std::path::PathBuf;

use crate::playlist_item::PlaylistItem;
use error_generator::error;
use rusqlite::Connection;

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
            uploader TEXT NOT NULL,
            previous_video_id NULL
        );", [])?;

        Ok(Database { connection })
    }

    fn get_path() -> PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("new_tube.db");
        path
    }

    pub fn query_all_items(&self) -> Result<Vec<PlaylistItem>> {
        let mut statement = self.connection.prepare("\
            SELECT * FROM PlaylistItems;
        ")?;

        let result = statement.query_map([], |row| {
            Ok(PlaylistItem {
                playlist_id: row.get(0)?,
                video_id: row.get(1)?,
                title: row.get(2)?,
                duration: row.get(3)?,
                uploader: row.get(4)?,
                previous_video_id: row.get(5)?
            })
        })?;
        Ok(result.map(|r| r.unwrap()).collect())
    }

    pub fn get_playlist_ids(&self) -> Result<Vec<String>> {
        Ok(self.query_all_items()?.into_iter().map(|item| item.playlist_id).collect())
    }

    pub fn add_item(&self, item: &PlaylistItem) -> Result<()> {
        self.connection.execute("\
            INSERT OR REPLACE INTO PlaylistItems (playlist_id, video_id, title, duration, uploader, previous_video_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6);
        ", (
            &item.playlist_id,
            &item.video_id,
            &item.title,
            &format!("{}", item.duration),
            &item.uploader,
            &item.previous_video_id
        ))?;

        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.connection.execute("DELETE FROM PlaylistItems WHERE playlist_id = ?1", &[id])?;
        Ok(())
    }
}

#[error(message = "Error while connecting to the database or while executing queries: {self.0}", impl_from)]
pub struct DBError(rusqlite::Error);