use clap::Parser;
use cli_table::table::Table;
use error_generator::error;

use Command::*;
use NewTubeError::*;

use crate::api::APICaller;
use crate::db::{Database, Playlist};
use crate::video::Video;

mod api;
mod video;
mod db;
mod date;

fn main() -> Result<(), NewTubeError> {
    match Command::parse() {
        Add(add_command) => add(&add_command.playlist_id),
        New => new(),
        NewJson => new_json(),
        Last => last(),
        PlaylistsJSON => unimplemented!()
    }
}

fn add(id: &str) -> Result<(), NewTubeError> {
    let database = Database::open()?;
    let latest_videos = APICaller::new()?.get_latest_videos(id)?;
    let playlist = match latest_videos.first() {
        Some(video) => Playlist::from((id, video)),
        None => return Err(PlaylistHasNoVideos)
    };

    database.add_playlist(playlist)?;
    Ok(())
}

fn new() -> Result<(), NewTubeError> {
    let new_videos = get_new_videos_and_update_database()?;

    Table::new()
        .header(["Channel", "Video", "Link", "Release Date"])
        .print_data(new_videos.iter());

    Ok(())
}

fn new_json() -> Result<(), NewTubeError> {
    let new_videos = get_new_videos_and_update_database()?;
    let new_videos_json = serde_json::to_string(&new_videos).unwrap();
    println!("{new_videos_json}");
    Ok(())
}

fn get_new_videos_and_update_database() -> Result<Vec<Video>, NewTubeError> {
    let database = Database::open()?;
    let api_caller = APICaller::new()?;
    let mut new_videos = vec![];

    for list in database.get_playlists()? {
        let list_id = &list.id;
        let last_video_release = &list.last_video_release;

        new_videos.extend(api_caller.get_latest_videos(list_id)?.into_iter().filter(|v| v.is_new(last_video_release)))
    }

    for video in &new_videos {
        database.update_playlist(video)?
    }

    Ok(new_videos)
}

fn last() -> Result<(), NewTubeError> {
    let database = Database::open()?;
    let videos: Vec<Video> = database.get_playlists()?
        .into_iter()
        .map(Playlist::into)
        .collect();

    Table::new()
        .header(["Channel", "Video", "Link", "Release Date"])
        .print_data(videos.iter());
    Ok(())
}

#[derive(Parser)]
enum Command {
    /// Add a playlist id
    Add(AddPlaylistCommand),
    /// Show the new videos of today
    New,
    /// Get the new videos as JSON
    #[clap(name = "new_json")]
    NewJson,
    /// Show the last video of every playlist. This does not call the Youtube API
    Last,
    /// Return all saved playlist IDs as JSON
    PlaylistsJSON,
}

#[derive(Parser)]
struct AddPlaylistCommand {
    /// The id of a "All Videos" playlist
    playlist_id: String,
}

#[error]
enum NewTubeError {
    #[error(message = "The playlist for the given id has no videos.")]
    PlaylistHasNoVideos,
    #[error(message = "Youtube API Call failed. Error: {_0}", impl_from)]
    ApiCallFailed(crate::api::ApiCallerError),
    #[error(message = "Database call failed. Error: {_0}", impl_from)]
    DatabaseCallFailed(crate::db::DBError),
}
