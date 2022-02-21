use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use cli_table::table::{Table, Width};
use error_generator::error;

use Command::*;
use NewTubeError::*;

use crate::db::{Database, Playlist};
use crate::video::Video;
use crate::video_retriever::VideoRetriever;

mod api;
mod video;
mod db;
mod video_retriever;
mod date_helper;

fn main() -> Result<(), NewTubeError> {
    match Command::parse() {
        Add(add_command) => add(&add_command.playlist_id),
        AddAll(add_all_command) => add_all(add_all_command.playlists_json_path),
        New => new(),
        NewJson => new_json(),
        Last => last(),
        PlaylistsJSON => playlists_json()
    }
}

fn add(id: &str) -> Result<(), NewTubeError> {
    let database = Database::open()?;
    let video_retriever = VideoRetriever::new()?;
    let latest_videos = video_retriever.get_latest_videos_for_playlist(id)?;
    let playlist = match latest_videos.first() {
        Some(video) => Playlist::from((id, video)),
        None => return Err(PlaylistHasNoVideos)
    };

    database.add_playlist(playlist)?;
    Ok(())
}

fn add_all(playlists_json_path: PathBuf) -> Result<(), NewTubeError> {
    let json_file = File::open(playlists_json_path).expect("open file");
    let ids: Vec<String> = serde_json::from_reader(json_file).expect("read file");

    ids.into_iter().for_each(|id| add(&id).unwrap());
    Ok(())
}

fn new() -> Result<(), NewTubeError> {
    let new_videos = get_new_videos_and_update_database()?;

    Table::new(|video: Video| [
        video.channel_name.clone(),
        video.name.clone(),
        video.link(),
        video.formatted_release_date(),
        video.duration
    ])
        .header(["Channel", "Video", "Link", "Release Date", "Duration"])
        .column_widths([Width::Dynamic, Width::Max(50), Width::Dynamic, Width::Dynamic, Width::Dynamic])
        .print(new_videos);

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
    let video_retriever = VideoRetriever::new()?;
    let mut new_videos = vec![];

    for list in database.get_playlists()? {
        let list_id = &list.id;
        let last_video_release = &list.last_video_release;

        new_videos.extend(video_retriever.get_new_videos_for_playlist(list_id, last_video_release)?)
    }

    for video in &new_videos {
        database.update_playlist(video)?
    }

    Ok(new_videos)
}

fn last() -> Result<(), NewTubeError> {
    let database = Database::open()?;
    let mut videos: Vec<Video> = database.get_playlists()?
        .into_iter()
        .map(Playlist::into)
        .collect();
    videos.sort_by(|v0, v1| v1.release_date.cmp(&v0.release_date));

    Table::new(|video: Video| [
        video.channel_name.clone(),
        video.name.clone(),
        video.link(),
        video.formatted_release_date(),
        video.duration
    ])
        .header(["Channel", "Video", "Link", "Release Date", "Duration"])
        .column_widths([Width::Dynamic, Width::Max(50), Width::Dynamic, Width::Dynamic, Width::Dynamic])
        .print(videos);
    Ok(())
}

fn playlists_json() -> Result<(), NewTubeError> {
    let database = Database::open()?;
    let playlist_ids = database.get_playlist_ids()?;
    let playlist_ids_json = serde_json::to_string(&playlist_ids).unwrap();
    println!("{playlist_ids_json}");
    Ok(())
}

#[derive(Parser)]
enum Command {
    /// Add a playlist id
    Add(AddCommand),
    /// Add a JSON list of playlists
    AddAll(AddAllCommand),
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
struct AddCommand {
    /// The id of a "All Videos" playlist
    playlist_id: String,
}

#[derive(Parser)]
struct AddAllCommand {
    /// Path to a JSON containing a list of playlist ids to add
    playlists_json_path: PathBuf
}

#[error]
enum NewTubeError {
    #[error(message = "The playlist for the given id has no videos.")]
    PlaylistHasNoVideos,
    #[error(message = "Video retrieval failed. Error: {_0}", impl_from)]
    VideoRetrieveFailed(crate::video_retriever::VideoRetrieveError),
    #[error(message = "Database call failed. Error: {_0}", impl_from)]
    DatabaseCallFailed(crate::db::DBError),
}