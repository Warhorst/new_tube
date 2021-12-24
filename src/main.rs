use crate::db::Database;
use clap::Parser;
use cli_table::table::Table;
use SubCommand::*;
use crate::api::APICaller;

mod api;
mod video;
mod db;

fn main() {
    let database = Database::open().unwrap();
    let newtube_command = SubCommand::parse();
    let api_caller = APICaller::new().unwrap();

    match newtube_command {
        AddPlaylist(command) => database.add_playlist(command.playlist_id).unwrap(),
        ShowNewVideos =>  {
            let new_videos = database.get_playlists().unwrap()
                .into_iter()
                .flat_map(|list| api_caller.get_latest_videos(&list.id).unwrap())
                .filter(|video| video.is_new())
                .collect::<Vec<_>>();

            Table::new()
                .header(["Channel", "Video", "Link", "Release Date"])
                .print_data(new_videos.iter())
        }
    }
}

#[derive(Parser)]
enum SubCommand {
    /// Add a playlist id
    AddPlaylist(AppPlaylistCommand),
    /// Show the new videos of today
    ShowNewVideos,
}

#[derive(Parser)]
struct AppPlaylistCommand {
    playlist_id: String,
}
