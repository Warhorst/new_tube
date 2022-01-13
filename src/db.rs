use std::convert::TryFrom;
use std::path::PathBuf;
use error_generator::error;
use rusqlite::{Connection, Row};
use crate::date::Date;
use crate::video::Video;

pub struct Database {
    connection: Connection
}

impl Database {
    pub fn open() -> Result<Self, DBError> {
        let connection = Connection::open(Self::get_path())?;

        connection.execute("\
        CREATE TABLE IF NOT EXISTS Playlists (
            id TEXT PRIMARY KEY,
            channel_name TEXT NOT NULL,
            last_video_name TEXT NOT NULL,
            last_video_id TEXT NOT NULL,
            last_video_release TEXT NOT NULL
        );", [])?;

        Ok(Database { connection })
    }

    fn get_path() -> PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("new_tube.db");
        path
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>, DBError> {
        let mut statement = self.connection.prepare("\
            SELECT * FROM Playlists;
        ")?;

        let result = statement.query_map([], |row| {
            Ok(Playlist::try_from(row)?)
        })?;
        Ok(result.map(|r| r.unwrap()).collect())
    }

    pub fn add_playlist(&self, playlist: Playlist) -> Result<(), DBError> {
        self.connection.execute("\
            INSERT INTO Playlists (id, channel_name, last_video_name, last_video_id, last_video_release)
            VALUES (?1, ?2, ?3, ?4, ?5);
        ", &[&playlist.id, &playlist.channel_name, &playlist.last_video_name, &playlist.last_video_id, &playlist.last_video_release])?;

        Ok(())
    }

    pub fn update_playlist(&self, id: &String, video: &Video) -> Result<(), DBError> {
        self.connection.execute("\
            UPDATE Playlists SET channel_name=?2, last_video_name=?3, last_video_id=?4, last_video_release=?5 WHERE id=?1;
        ", &[&id, &video.channel_name, &video.name, &video.id, &video.release_date.to_db_playlist_date()])?;

        Ok(())
    }
}

pub struct Playlist {
    pub id: String,
    pub channel_name: String,
    pub last_video_name: String,
    pub last_video_id: String,
    pub last_video_release: String
}

impl TryFrom<&Row<'_>> for Playlist {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Playlist {
            id: row.get(0)?,
            channel_name: row.get(1)?,
            last_video_name: row.get(2)?,
            last_video_id: row.get(3)?,
            last_video_release: row.get(4)?
        })
    }
}

impl From<(&String, &Video)> for Playlist {
    fn from(id_video: (&String, &Video)) -> Self {
        let playlist_id = id_video.0;
        let video = id_video.1;

        Playlist {
            id: playlist_id.clone(),
            channel_name: video.channel_name.clone(),
            last_video_name: video.name.clone(),
            last_video_id: video.id.clone(),
            last_video_release: video.release_date.to_db_playlist_date()
        }
    }
}

impl Into<Video> for Playlist {
    fn into(self) -> Video {
        Video {
            id: self.last_video_id,
            name: self.last_video_name,
            channel_name: self.channel_name,
            release_date: Date::from_db_playlist_date(&self.last_video_release)
        }
    }
}

#[error(message = "Error while connecting to the database or while executing queries: {0}", impl_from)]
pub struct DBError(rusqlite::Error);