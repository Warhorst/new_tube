use error_generator::error;

use ApiCallerError::*;
use playlist_items::PlaylistItems;

use crate::video::Video;

mod playlist_items;

type PlaylistId<'a> = &'a str;

pub struct APICaller {
    api_key: String,
}

impl APICaller {
    const YOUTUBE_API_KEY: &'static str = "YOUTUBE_API_KEY";
    const MAX_RESULTS: u8 = 3;

    pub fn new() -> Result<Self, ApiCallerError> {
        let api_key = std::env::var(Self::YOUTUBE_API_KEY)?;
        Ok(APICaller { api_key })
    }

    /// Return the latest youtube videos from the given playlist id. The videos are
    /// fetched using the youtube data API v3 PlaylistItems method.
    pub fn get_latest_videos(&self, id: PlaylistId) -> Result<Vec<Video>, ApiCallerError> {
        let url = self.create_url(id);
        let playlist_items_json = self.get_playlist_items_as_json(url)?;
        let items_response: PlaylistItems = serde_json::from_str(&playlist_items_json)?;
        Ok(items_response.into())
    }

    /// Create the url to the playlist item api (see https://developers.google.com/youtube/v3/docs/playlistItems/list).
    fn create_url(&self, id: PlaylistId) -> String {
        format!(
            "https://www.googleapis.com/youtube/v3/playlistItems?\
            part=snippet\
            &maxResults={}\
            &playlistId={}\
            &key={}",
            Self::MAX_RESULTS,
            id,
            self.api_key
        )
    }

    fn get_playlist_items_as_json(&self, url: String) -> Result<String, ApiCallerError> {
        // TODO don't be stupid and use futures.
        let response = reqwest::blocking::get(url)?;
        match response.status().as_u16() {
            200 => Ok(response.text()?),
            code => Err(ErrorResponse(code, response.text()?))
        }
    }
}

#[error]
pub enum ApiCallerError {
    #[error(message = "The system variable 'YOUTUBE_API_KEY' is not set", impl_from)]
    APIKeyNotSet(std::env::VarError),
    #[error(message = "Error while requesting the playlist items from youtube: {_0}", impl_from)]
    RequestFailed(reqwest::Error),
    #[error(message = "The request returned code {_0}. Response: {_1}")]
    ErrorResponse(u16, String),
    #[error(message = "The response could not be parsed. Error: {_0}", impl_from)]
    ParsingFailed(serde_json::Error)
}
