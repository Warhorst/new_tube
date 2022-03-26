use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use cli_table::table::{Table, Width};
use error_generator::error;

use Command::*;
use NewTubeError::*;

use crate::date_helper::compare_video_releases;
use crate::db::Database;
use crate::playlist::Playlist;
use crate::video::Video;
use crate::video_retriever::VideoRetriever;

mod api;
mod video;
mod db;
mod video_retriever;
mod date_helper;
mod duration_formatter;
mod playlist;

type Result<T> = std::result::Result<T, NewTubeError>;

fn main() -> Result<()> {
    match Command::parse() {
        Add(add_command) => add(&add_command.playlist_id),
        AddAll(add_all_command) => add_all(add_all_command.playlists_json_path),
        New => new(),
        NewJson => new_json(),
        Last => last(),
        PlaylistsJSON => playlists_json(),
        DbDebug => db_debug()
    }
}

fn add(id: &str) -> Result<()> {
    let database = Database::open()?;
    let video_retriever = VideoRetriever::new()?;
    let latest_videos = video_retriever.get_latest_videos_for_playlist(id)?;
    let playlist = match latest_videos.iter().max_by(|v0, v1| compare_video_releases(v0, v1)) {
        Some(video) => Playlist::from((id, video)),
        None => return Err(PlaylistHasNoVideos)
    };

    database.add_playlist(playlist)?;
    Ok(())
}

fn add_all(playlists_json_path: PathBuf) -> Result<()> {
    let json_file = File::open(playlists_json_path).expect("open file");
    let ids: Vec<String> = serde_json::from_reader(json_file).expect("read file");

    ids.into_iter().for_each(|id| add(&id).unwrap());
    Ok(())
}

fn new() -> Result<()> {
    let new_videos = get_new_videos_and_update_database()?;
    print_table(new_videos);
    Ok(())
}

fn new_json() -> Result<()> {
    let new_videos = get_new_videos_and_update_database()?;
    let new_videos_json = serde_json::to_string(&new_videos).unwrap();
    println!("{new_videos_json}");
    Ok(())
}

fn get_new_videos_and_update_database() -> Result<Vec<Video>> {
    let database = Database::open()?;
    let retriever = VideoRetriever::new()?;
    let playlist_ids_with_timestamp = get_playlist_ids_with_timestamp_from_db(&database)?;
    let videos = retriever.get_new_videos_for_playlists(playlist_ids_with_timestamp)?;
    update_playlists_in_db(&database, &videos)?;
    Ok(videos)
}

fn get_playlist_ids_with_timestamp_from_db(database: &Database) -> Result<Vec<(String, String)>> {
    let playlist_ids_with_timestamps = database.get_playlists()?
        .into_iter()
        .map(|p| (p.id.clone(), p.last_video_release.clone()))
        .collect();
    Ok(playlist_ids_with_timestamps)
}

fn update_playlists_in_db(database: &Database, videos: &Vec<Video>) -> Result<()> {
    let mut playlist_id_video_map = HashMap::new();

    for video in videos {
        let entry = playlist_id_video_map.entry(video.playlist_id.clone()).or_insert(vec![]);
        entry.push(video);
    }

    for videos in playlist_id_video_map.values() {
        let latest_video = videos.into_iter().max_by(|v0, v1| compare_video_releases(v0, v1)).unwrap();
        database.update_playlist(latest_video)?;
    }

    Ok(())
}

fn last() -> Result<()> {
    let database = Database::open()?;
    let mut videos: Vec<Video> = database.get_playlists()?
        .into_iter()
        .map(Playlist::into)
        .collect();
    videos.sort_by(|v0, v1| v1.release_date.cmp(&v0.release_date));
    print_table(videos);
    Ok(())
}

fn playlists_json() -> Result<()> {
    let database = Database::open()?;
    let playlist_ids = database.get_playlist_ids()?;
    let playlist_ids_json = serde_json::to_string(&playlist_ids).unwrap();
    println!("{playlist_ids_json}");
    Ok(())
}

fn db_debug() -> Result<()> {
    Ok(Database::open()?.print_debug_info()?)
}

fn print_table(videos: Vec<Video>) {
    Table::new(|video: Video| [
        video.channel_name.clone(),
        video.name.clone(),
        video.link(),
        video.formatted_release_date(),
        video.formatted_duration()
    ])
        .header(["Channel", "Video", "Link", "Release Date", "Duration"])
        .column_widths([Width::Dynamic, Width::Max(50), Width::Dynamic, Width::Dynamic, Width::Dynamic])
        .print(videos);
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
    /// Print all database rows of the saved videos.
    DbDebug
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