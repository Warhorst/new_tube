use serde::Deserialize;

use crate::video::Video;

/// Represents a successful response from youtube when sending a request to playlistItems/list of the Youtube data API.
#[derive(Debug, Deserialize)]
pub struct PlaylistItems {
    pub items: Vec<PlaylistItem>
}

#[derive(Debug, Deserialize)]
pub struct PlaylistItem {
    pub snippet: Snippet,
    #[serde(rename(deserialize = "contentDetails"))]
    pub content_details: ContentDetails
}

#[derive(Debug, Deserialize)]
pub struct Snippet {
    pub title: String,
    #[serde(rename(deserialize = "channelTitle"))]
    pub channel_title: String,
    #[serde(rename(deserialize = "playlistId"))]
    pub playlist_id: String,
    #[serde(rename(deserialize = "resourceId"))]
    pub resource_id: ResourceId
}

#[derive(Debug, Deserialize)]
pub struct ContentDetails {
    #[serde(rename(deserialize = "videoPublishedAt"))]
    pub video_published_at: String
}

#[derive(Debug, Deserialize)]
pub struct ResourceId {
    #[serde(rename(deserialize = "videoId"))]
    pub video_id: String
}

impl Into<Vec<Video>> for PlaylistItems {
    fn into(self) -> Vec<Video> {
        self.items
            .into_iter()
            .map(|item| Video::new(
                item.snippet.playlist_id,
                item.snippet.channel_title,
                item.snippet.title,
                item.snippet.resource_id.video_id,
                item.content_details.video_published_at
            )).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::playlist_items::{ContentDetails, PlaylistItem, PlaylistItems, ResourceId, Snippet};
    use crate::video::Video;

    #[test]
    fn into_videos_works() {
        let items = PlaylistItems {
            items: vec![
                PlaylistItem {
                    snippet: Snippet {
                        title: "V0".to_string(),
                        channel_title: "C0".to_string(),
                        playlist_id: "CID0".to_string(),
                        resource_id: ResourceId {
                            video_id: "I0".to_string()
                        }
                    },
                    content_details: ContentDetails {
                        video_published_at: "2021-12-18T19:13:00Z".to_string()
                    }
                },
                PlaylistItem {
                    snippet: Snippet {
                        title: "V1".to_string(),
                        channel_title: "C1".to_string(),
                        playlist_id: "CID1".to_string(),
                        resource_id: ResourceId {
                            video_id: "I1".to_string()
                        }
                    },
                    content_details: ContentDetails {
                        video_published_at: "2021-12-18T19:13:00Z".to_string()
                    }
                }
            ]
        };
        let _videos: Vec<Video> = items.into();
    }
}