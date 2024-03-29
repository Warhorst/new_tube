use std::{env, io};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;
use cli_table::table::{Table, Width};
use error_generator::error;
use ron::error::SpannedError;

use Command::*;

use crate::config::Config;
use crate::new_tube_service::database::Database;
use crate::new_tube_service::NewTubeService;
use crate::new_tube_service::yt_dlp::{Item, Items};
use crate::telegram_bot::Bot;

mod environment;
mod new_tube_service;
mod telegram_bot;
mod config;

type Result<T> = std::result::Result<T, NewTubeError>;

fn main() -> Result<()> {
    let config = load_config()?;

    match Command::parse() {
        Add(add_command) => add(&add_command.playlist_id),
        AddAll(add_all_command) => add_all(add_all_command.playlists_json_path),
        New => new(),
        Last => last(),
        PlaylistsJSON => playlists_json(),
        Command::Bot => Ok(Bot::run(config)?),
        Replace(replace_command) => replace(&replace_command.old_playlist_id, &replace_command.new_playlist_id),
        Delete(delete_command) => delete(&delete_command.playlist_id)
    }
}

fn load_config() -> Result<Config> {
    let exe_path = env::current_exe()?;
    let config_path = exe_path.parent().expect("parent directory should exist").join("config.ron");

    let mut buf = String::new();
    File::open(config_path)?.read_to_string(&mut buf)?;
    let config = ron::from_str::<Config>(&buf)?;
    Ok(config)
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

fn replace(old_id: &str, new_id: &str) -> Result<()> {
    let service = NewTubeService::new()?;
    service.replace(old_id, new_id)?;
    Ok(())
}

fn delete(id: &str) -> Result<()> {
    let service = NewTubeService::new()?;
    service.delete(id)?;
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
        item.playlist_id.clone(),
        item.title.clone(),
        item.link(),
        item.formatted_duration(),
    ])
        .header(["Channel", "Playlist ID", "Video", "Link", "Duration"])
        .column_widths([Width::Dynamic, Width::Dynamic, Width::Max(50), Width::Dynamic, Width::Dynamic])
        .print(items);
}

#[derive(Parser)]
enum Command {
    /// Add a playlist id
    Add(AddCommand),
    /// Add a JSON list of playlists
    AddAll(AddAllCommand),
    /// Replace an existing playlist id with a new one
    Replace(ReplaceCommand),
    /// Delete an existing playlist id
    Delete(DeleteCommand),
    /// Show the new videos of today
    New,
    /// Show the last video of every playlist. This does not call the Youtube API
    Last,
    /// Return all saved playlist IDs as JSON
    PlaylistsJSON,
    /// Run the telegram bot. Requires NEW_TUBE_TELEGRAM_API_KEY to be set to the
    /// telegram API key. Updates will be sent to the channel defined by NEW_TUBE_DEFAULT_TELEGRAM_CHANNEL.
    /// Only the user defined in NEW_TUBE_ALLOWED_BOT_USER can use the bot.
    Bot
}

#[derive(Parser)]
struct AddCommand {
    /// The id of an "All Videos" playlist
    playlist_id: String,
}

#[derive(Parser)]
struct AddAllCommand {
    /// Path to a JSON containing a list of playlist ids to add
    playlists_json_path: PathBuf,
}

#[derive(Parser)]
struct ReplaceCommand {
    /// The old id to be replaced
    old_playlist_id: String,
    /// The new id to add to new_tube
    new_playlist_id: String,
}

#[derive(Parser)]
struct DeleteCommand {
    /// The playlist id of the playlist id to be deleted
    playlist_id: String,
}

#[error]
enum NewTubeError {
    #[error(message = "IO error: {_0}", impl_from)]
    IoError(io::Error),
    #[error(message = "Failed to parse config file: {_0}", impl_from)]
    ConfigParseError(SpannedError),
    #[error(message = "{_0}", impl_from)]
    VideoServiceError(new_tube_service::NewTubeServiceError),
    #[error(message = "Database call failed. Error: {_0}", impl_from)]
    DatabaseCallFailed(new_tube_service::database::DBError),
    #[error(message = "{_0}", impl_from)]
    BotError(telegram_bot::BotError),
}