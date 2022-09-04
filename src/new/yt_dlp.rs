use std::process::{Command, Output};
use std::string::FromUtf8Error;

use error_generator::error;
use rusqlite::Row;
use serde::Deserialize;

pub type Items = Vec<Item>;
type Result<T> = std::result::Result<T, Error>;

pub struct YTDLPCaller;

impl YTDLPCaller {
    /// Use yt-dlp to retrieve the last 3 items from the given playlist id and parse them.
    pub fn retrieve_latest_items(playlist_id: &str) -> Result<Items> {
        let output = Self::execute_command(playlist_id)?;
        Self::parse_output_to_items(output)
    }

    fn execute_command(playlist_id: &str) -> Result<Output> {
        // TODO: There is an async process library, but it only works blocking on windows. Could be faster if run
        //  on a penguin machine.
        Ok(Command::new("yt-dlp")
            .arg(&format!("https://www.youtube.com/watch?list={}", playlist_id))
            .arg("--skip-download")
            .arg("--quiet")
            .arg("--playlist-start")
            .arg("1")
            .arg("--playlist-end")
            .arg("3")
            .arg("--print-json")
            .arg("--flat-playlist")
            .output()?)
    }

    fn parse_output_to_items(output: Output) -> Result<Items> {
        let output_string = String::from_utf8(output.stdout)?;
        let mut items = vec![];

        for s in output_string.split("\n").into_iter().filter(|s| !s.is_empty()) {
            items.push(serde_json::from_str(s)?)
        }

        Ok(items)
    }
}

#[error]
pub enum Error {
    #[error(message = "Failed to execute the youtube-dlp process. Reason: {_0}", impl_from)]
    ProcessFailed(std::io::Error),
    #[error(message = "Failed to parse the process output to items. Reason: {_0}", impl_from)]
    ItemParseFailed(serde_json::Error),
    #[error(message = "stdout from output could not be parsed: Reason: {_0}", impl_from)]
    CommandOutputParseFailed(FromUtf8Error)
}

#[derive(Clone, Debug, Deserialize)]
pub struct Item {
    pub playlist_id: String,
    #[serde(rename(deserialize = "id"))]
    pub video_id: String,
    pub title: String,
    pub duration: f32,
    pub uploader: String
}

impl Item {
    pub fn link(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.video_id)
    }

    pub fn formatted_duration(&self) -> String {
        let secs = self.duration as usize;
        let seconds = secs % 60;
        let minutes = (secs / 60) % 60;
        let hours = (secs / 60) / 60;
        format!("{hours}:{minutes}:{seconds}")
    }
}

impl TryFrom<&Row<'_>> for Item {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> std::result::Result<Self, Self::Error> {
        Ok(Item {
            playlist_id: row.get(0)?,
            video_id: row.get(1)?,
            title: row.get(2)?,
            duration: row.get(3)?,
            uploader: row.get(4)?,
        })
    }
}