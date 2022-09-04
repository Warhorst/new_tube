use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use cli_table::table::{Table, Width};
use error_generator::error;

use Command::*;

use crate::new::database::Database;
use crate::new::NewTubeService;
use crate::new::yt_dlp::{Item, Items};
use crate::telegram::bot::Bot;

mod telegram;
mod environment;
mod new;

type Result<T> = std::result::Result<T, NewTubeError>;

#[tokio::main]
async fn main() -> Result<()> {
    match Command::parse() {
        Add(add_command) => add(&add_command.playlist_id),
        AddAll(add_all_command) => add_all(add_all_command.playlists_json_path),
        New => new(),
        Last => last(),
        PlaylistsJSON => playlists_json(),
        Bot => Ok(Bot::new()?.run().await?)
    }
}

fn add(id: &str) -> Result<()> {
    let video_service = NewTubeService::new()?;
    Ok(video_service.add_playlist(id)?)
}

fn add_all(playlists_json_path: PathBuf) -> Result<()> {
    let json_file = File::open(playlists_json_path).expect("open file");
    let ids: Vec<String> = serde_json::from_reader(json_file).expect("read file");

    for id in ids {
        add(&id)?
    }

    Ok(())
}

fn new() -> Result<()> {
    let video_service = NewTubeService::new()?;
    let new_items = video_service.get_new_videos_and_update_database()?;
    print_table(new_items);
    Ok(())
}

fn last() -> Result<()> {
    let database = Database::open()?;
    let items = database.get_items()?;
    print_table(items);
    Ok(())
}

fn playlists_json() -> Result<()> {
    let database = Database::open()?;
    let playlist_ids = database.get_playlist_ids()?;
    let playlist_ids_json = serde_json::to_string(&playlist_ids).unwrap();
    println!("{playlist_ids_json}");
    Ok(())
}

fn print_table(items: Items) {
    Table::new(|item: Item| [
        item.uploader.clone(),
        item.title.clone(),
        item.link(),
        item.formatted_duration(),
    ])
        .header(["Channel", "Video", "Link", "Duration"])
        .column_widths([Width::Dynamic, Width::Max(50), Width::Dynamic, Width::Dynamic])
        .print(items);
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
    VideoServiceError(new::NewTubeServiceError),
    #[error(message = "Database call failed. Error: {_0}", impl_from)]
    DatabaseCallFailed(new::database::DBError),
    #[error(message = "{_0}", impl_from)]
    BotError(telegram::bot::BotError)
}