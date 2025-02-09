use crate::new_tube_service;
use crate::new_tube_service::database::Database;
use error_generator::error;
use std::fs::File;
use std::path::PathBuf;
use std::{env, io};

const FILE_NAME: &'static str = "playlists.json";

/// Write all the current playlist ids into a local file
pub fn dump_playlist_ids() -> Result<(), DumpError> {
    let path = create_path()?;
    let dump_file = File::create(path)?;

    let database = Database::open()?;
    let playlist_ids = database.get_playlist_ids()?;
    Ok(serde_json::to_writer(dump_file, &playlist_ids)?)
}

pub fn load_playlists_dump() -> Result<Vec<String>, DumpError> {
    let path = create_path()?;
    let dump_file = File::open(path)?;

    let playlist_ids: Vec<String> = serde_json::from_reader(dump_file)?;

    Ok(playlist_ids)
}

fn create_path() -> Result<PathBuf, DumpError> {
    let exe_path = env::current_exe()?;
    Ok(exe_path.parent().expect("parent directory should exist").join(FILE_NAME))
}

#[error]
pub enum DumpError {
    #[error(message = "IO error: {_0}", impl_from)]
    IoError(io::Error),
    #[error(message = "Database call failed. Error: {_0}", impl_from)]
    DatabaseCallFailed(new_tube_service::database::DBError),
    #[error(message = "Serialization/Deserialization failed. Error: {_0}", impl_from)]
    SerdeError(serde_json::Error)
}