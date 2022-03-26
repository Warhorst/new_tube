use std::convert::TryFrom;
use std::path::PathBuf;

use error_generator::error;
use rusqlite::{Connection, Row, Statement};
use crate::Playlist;

use crate::video::Video;

type Result<T> = std::result::Result<T, DBError>;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let connection = Connection::open(Self::get_path())?;

        connection.execute("\
        CREATE TABLE IF NOT EXISTS Playlists (
            id TEXT PRIMARY KEY,
            channel_name TEXT NOT NULL,
            last_video_name TEXT NOT NULL,
            last_video_id TEXT NOT NULL,
            last_video_release TEXT NOT NULL,
            last_video_duration TEXT NOT NULL
        );", [])?;

        Ok(Database { connection })
    }

    fn get_path() -> PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("new_tube.db");
        path
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>> {
        let mut statement = self.create_select_all_statement()?;

        let result = statement.query_map([], |row| {
            Playlist::try_from(row)
        })?;
        Ok(result.map(|r| r.unwrap()).collect())
    }

    pub fn get_playlist_ids(&self) -> Result<Vec<String>> {
        Ok(self.get_playlists()?.into_iter().map(|p| p.id).collect())
    }

    pub fn add_playlist(&self, playlist: Playlist) -> Result<()> {
        self.connection.execute("\
            INSERT INTO Playlists (id, channel_name, last_video_name, last_video_id, last_video_release, last_video_duration)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6);
        ", &[&playlist.id, &playlist.channel_name, &playlist.last_video_name, &playlist.last_video_id, &playlist.last_video_release, &playlist.last_video_duration])?;

        Ok(())
    }

    pub fn update_playlist(&self, video: &Video) -> Result<()> {
        self.connection.execute("\
            UPDATE Playlists SET channel_name=?2, last_video_name=?3, last_video_id=?4, last_video_release=?5, last_video_duration=?6 WHERE id=?1;
        ", &[&video.playlist_id, &video.channel_name, &video.name, &video.id, &video.release_date, &video.duration])?;

        Ok(())
    }

    pub fn print_debug_info(&self) -> Result<()> {
        let mut statement = self.create_select_all_statement()?;

        statement.query_map([], |row| {
            let column_names = row.column_names();
            let result = (0..column_names.len()).into_iter()
                .fold(String::new(), |acc, i| Self::row_to_string(acc, i, row));
            Ok(result)
        })?.for_each(|entry_str| println!("{}", entry_str.unwrap()));

        Ok(())
    }

    fn row_to_string(mut acc: String, i: usize, row: &Row) -> String {
        acc.push_str(row.column_name(i).unwrap());
        acc.push_str(": ");
        acc.push_str(&row.get::<_, String>(i).unwrap());
        acc.push_str("\n");
        acc
    }

    fn create_select_all_statement(&self) -> Result<Statement> {
        Ok(self.connection.prepare("\
            SELECT * FROM Playlists;
        ")?)
    }
}

#[error(message = "Error while connecting to the database or while executing queries: {self.0}", impl_from)]
pub struct DBError(rusqlite::Error);