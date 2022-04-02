use error_generator::error;
use serde::de::DeserializeOwned;

use crate::api::caller::ApiCallerError::ErrorResponse;
use crate::api::playlist_items::PlaylistItems;
use crate::api::video_items::VideoItems;

pub type Result<T> = std::result::Result<T, ApiCallerError>;

pub struct APICaller {
    api_key: String,
}

impl APICaller {
    const YOUTUBE_API_KEY: &'static str = "YOUTUBE_API_KEY";

    pub fn new() -> Result<Self> {
        let api_key = std::env::var(Self::YOUTUBE_API_KEY)?;
        Ok(APICaller { api_key })
    }

    pub async fn get_playlist_items(&self, id: &str) -> Result<PlaylistItems> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/playlistItems?\
            part=snippet,contentDetails\
            &maxResults=3\
            &playlistId={}\
            &key={}",
            id,
            self.api_key
        );

        self.get(&url).await
    }

    pub async fn get_video_items(&self, ids: Vec<&str>) -> Result<VideoItems> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?\
            part=contentDetails\
            &id={}\
            &key={}",
            ids.iter().enumerate().fold(String::new(), |mut acc, (i, id)| {
                let append = if i != ids.len() - 1 {format!("{id},")} else {id.to_string()};
                acc += &append;
                acc
            }),
            self.api_key
        );

        self.get(&url).await
    }

    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = reqwest::get(url).await?;
        let body = match response.status().as_u16() {
            200 => response.text().await?,
            code => return Err(ErrorResponse(code, response.text().await?))
        };
        let value = serde_json::from_str(&body)?;
        Ok(value)
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
    ParsingFailed(serde_json::Error),
}