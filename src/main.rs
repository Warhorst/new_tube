use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use cli_table::table::{Table, Width};
use error_generator::error;

use Command::*;

use crate::date_helper::compare_video_releases;
use crate::db::Database;
use crate::new_tube_service::NewTubeService;
use crate::playlist::Playlist;
use crate::telegram::bot::Bot;
use crate::video::Video;
use crate::video_retriever::VideoRetriever;

mod api;
mod video;
mod db;
mod video_retriever;
mod date_helper;
mod duration_formatter;
mod playlist;
mod telegram;
mod new_tube_service;
mod environment;

type Result<T> = std::result::Result<T, NewTubeError>;

#[tokio::main]
async fn main() -> Result<()> {
    match Command::parse() {
        Add(add_command) => add(&add_command.playlist_id).await,
        AddAll(add_all_command) => add_all(add_all_command.playlists_json_path).await,
        New => new().await,
        Last => last(),
        PlaylistsJSON => playlists_json(),
        DbDebug => db_debug(),
        Bot => Ok(Bot::new()?.run().await?)
    }
}

async fn add(id: &str) -> Result<()> {
    let video_service = NewTubeService::new()?;
    Ok(video_service.add_playlist(id).await?)
}

async fn add_all(playlists_json_path: PathBuf) -> Result<()> {
    let json_file = File::open(playlists_json_path).expect("open file");
    let ids: Vec<String> = serde_json::from_reader(json_file).expect("read file");

    for id in ids {
        add(&id).await?
    }

    Ok(())
}

async fn new() -> Result<()> {
    let video_service = NewTubeService::new()?;
    let new_videos = video_service.get_new_videos_and_update_database().await?;
    print_table(new_videos);
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
    /// Show the last video of every playlist. This does not call the Youtube API
    Last,
    /// Return all saved playlist IDs as JSON
    PlaylistsJSON,
    /// Print all database rows of the saved videos.
    DbDebug,
    /// Run the telegram
    Bot
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
    #[error(message = "{_0}", impl_from)]
    VideoServiceError(crate::new_tube_service::NewTubeServiceError),
    #[error(message = "Database call failed. Error: {_0}", impl_from)]
    DatabaseCallFailed(crate::db::DBError),
    #[error(message = "{_0}", impl_from)]
    BotError(crate::telegram::bot::BotError)
}