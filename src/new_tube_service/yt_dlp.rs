use std::process::{Command, Output};
use std::string::FromUtf8Error;

use error_generator::error;
use serde::Deserialize;

type Result<T> = std::result::Result<T, Error>;

/// Use yt-dlp to retrieve the last 2 items from the given playlist id and parse them.
pub fn retrieve_latest_items(playlist_id: &str) -> Result<YTDLPResponse> {
    let output = execute_command(playlist_id)?;
    parse_output_to_items(output)
}

// Example: yt-dlp https://www.youtube.com/watch?list=<PLAYLIST_ID> --skip-download --quiet --playlist-start 1 --playlist-end 3 --print-json --flat-playlist
fn execute_command(playlist_id: &str) -> Result<Output> {
    // TODO: There is an async process library, but it only works blocking on windows. Could be faster if run
    //  on a penguin machine.
    Ok(Command::new("yt-dlp")
        .arg(&format!(
            "https://www.youtube.com/watch?list={}",
            playlist_id
        ))
        .arg("--skip-download")
        .arg("--quiet")
        .arg("--playlist-start")
        .arg("1")
        .arg("--playlist-end")
        .arg("2")
        .arg("--print-json")
        .arg("--flat-playlist")
        .output()?)
}

fn parse_output_to_items(output: Output) -> Result<YTDLPResponse> {
    let output_string = String::from_utf8(output.stdout)?;
    let mut items = vec![];

    for s in output_string
        .split("\n")
        .into_iter()
        .filter(|s| !s.is_empty())
    {
        items.push(serde_json::from_str(s)?)
    }

    if items.len() != 2 {
        // The playlist needs at least 2 items. A latest one and the previous one
        return Err(Error::WrongAmountReturned)
    }

    let previous_item = items.remove(1);
    let latest_item = items.remove(0);

    Ok(YTDLPResponse {
        previous_item,
        latest_item,
    })
}

/// The response from a call to yt_dlp to retrieve the latest 2 videos from a playlist.
pub struct YTDLPResponse {
    /// The latest item from the playlist
    pub latest_item: YTDLPItem,
    /// The item previous to the latest item in the playlist
    pub previous_item: YTDLPItem
}

/// The relevant fields of a playlist item, returned from yt-dlp
#[derive(Clone, Debug, Deserialize)]
pub struct YTDLPItem {
    /// The playlist id
    pub playlist_id: String,
    /// The video id
    pub id: String,
    /// The video title
    pub title: String,
    /// The video duration in seconds. Might be None if
    /// the video is a running livestream
    pub duration: Option<f32>,
    /// The channel which uploaded the video
    pub channel: String,
}

#[error]
pub enum Error {
    #[error(
        message = "Failed to execute the youtube-dlp process. Reason: {_0}",
        impl_from
    )]
    ProcessFailed(std::io::Error),
    #[error(
        message = "Failed to parse the process output to items. Reason: {_0}",
        impl_from
    )]
    ItemParseFailed(serde_json::Error),
    #[error(
        message = "stdout from output could not be parsed: Reason: {_0}",
        impl_from
    )]
    CommandOutputParseFailed(FromUtf8Error),
    #[error(
        message = "yt_dlp did not return exactly 2 items, which is the expected amount"
    )]
    WrongAmountReturned
}
