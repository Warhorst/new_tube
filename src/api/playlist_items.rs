use serde::Deserialize;

/// Represents a successful response from youtube when sending a request to playlistItems/list of the Youtube data API.
#[derive(Debug, Deserialize)]
pub struct PlaylistItems {
    pub items: Vec<PlaylistItem>,
}

impl PlaylistItems {
    pub fn empty() -> Self {
        PlaylistItems { items: vec![] }
    }

    pub fn get_video_ids(&self) -> Vec<&str> {
        self.items.iter()
            .map(|item| item.snippet.resource_id.video_id.as_str())
            .collect()
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.items.extend(other.items);
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct PlaylistItem {
    pub snippet: Snippet,
    #[serde(rename(deserialize = "contentDetails"))]
    pub content_details: ContentDetails,
}

#[derive(Debug, Deserialize)]
pub struct Snippet {
    pub title: String,
    #[serde(rename(deserialize = "channelTitle"))]
    pub channel_title: String,
    #[serde(rename(deserialize = "playlistId"))]
    pub playlist_id: String,
    #[serde(rename(deserialize = "resourceId"))]
    pub resource_id: ResourceId,
}

#[derive(Debug, Deserialize)]
pub struct ContentDetails {
    #[serde(rename(deserialize = "videoPublishedAt"))]
    pub video_published_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ResourceId {
    #[serde(rename(deserialize = "videoId"))]
    pub video_id: String,
}