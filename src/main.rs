use error_gen::error;
use crate::db::{Database, Playlist};
use clap::Parser;
use cli_table::table::Table;
use Command::*;
use crate::api::APICaller;
use NewTubeError::*;
use crate::video::Video;

mod api;
mod video;
mod db;
mod date;

fn main() -> Result<(), NewTubeError> {
    let database = Database::open().unwrap();
    let command = Command::parse();
    let api_caller = APICaller::new().unwrap();

    match command {
        AddPlaylist(add_command) =>  {
            let latest_videos = APICaller::new()?.get_latest_videos(&add_command.playlist_id)?;
            let playlist = match latest_videos.first() {
                Some(video) => Playlist::from((&add_command.playlist_id, video)),
                None => return Err(PlaylistHasNoVideos)
            };

            database.add_playlist(playlist)?;
            Ok(())
        }
        ShowNewVideos =>  {
            let mut playlist_id_and_new_videos = vec![];
            for list in database.get_playlists()? {
                let list_id = &list.id;
                let last_video_release = &list.last_video_release;

                playlist_id_and_new_videos.extend(api_caller.get_latest_videos(&list.id)?
                    .into_iter()
                    .map(|v| (list_id.clone(), v))
                    .filter(|(_, v)| v.is_new(last_video_release)))
            }

            for (playlist_id, video) in &playlist_id_and_new_videos {
                database.update_playlist(playlist_id, video)?
            }

            Table::new()
                .header(["Channel", "Video", "Link", "Release Date"])
                .print_data(playlist_id_and_new_videos.iter().map(|(_, v)| v));

            Ok(())
        },
        GetLastVideos => {
            let videos: Vec<Video> = database.get_playlists()?
                .into_iter()
                .map(Playlist::into)
                .collect();

            Table::new()
                .header(["Channel", "Video", "Link", "Release Date"])
                .print_data(videos.iter());
            Ok(())
        }
    }
}

#[derive(Parser)]
enum Command {
    /// Add a playlist id
    AddPlaylist(AddPlaylistCommand),
    /// Show the new videos of today
    ShowNewVideos,
    /// Show the last video of every playlist. This does not call the Youtube API
    GetLastVideos
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
    #[error(message = "Youtube API Call failed. Error: {0}", impl_from)]
    ApiCallFailed(crate::api::ApiCallerError),
    #[error(message = "Database call failed. Error: {0}", impl_from)]
    DatabaseCallFailed(crate::db::DBError)
}
