use std::path::PathBuf;
use error_gen::error;
use rusqlite::Connection;

pub struct Database {
    connection: Connection
}

impl Database {
    pub fn open() -> Result<Self, DBError> {
        let connection = Connection::open(Self::get_path())?;

        connection.execute("\
        CREATE TABLE IF NOT EXISTS Playlists (
            id TEXT PRIMARY KEY
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
            SELECT id FROM Playlists;
        ")?;

        let result = statement.query_map([], |row| {
            Ok(Playlist {id: row.get(0).unwrap()})
        })?;
        Ok(result.map(|r| r.unwrap()).collect())
    }

    pub fn add_playlist(&self, playlist_id: String) -> Result<(), DBError> {
        self.connection.execute("\
            INSERT INTO Playlists (id)
            VALUES (?1);
        ", &[&playlist_id])?;

        Ok(())
    }
}

pub struct Playlist {
    pub id: String
}

#[error(message = "Error while connecting to the database or while executing queries: {0}", impl_from)]
pub struct DBError(rusqlite::Error);