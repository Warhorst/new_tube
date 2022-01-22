use serde::Deserialize;

use crate::video::Video;

/// Represents a successful response from youtube when sending a request to playlistItems/list of the Youtube data API.
#[derive(Debug, Deserialize)]
pub struct PlaylistItems {
    pub items: Vec<PlaylistItem>
}

#[derive(Debug, Deserialize)]
pub struct PlaylistItem {
    pub snippet: Snippet
}

#[derive(Debug, Deserialize)]
pub struct Snippet {
    #[serde(rename(deserialize = "publishedAt"))]
    pub published_at: String,
    pub title: String,
    #[serde(rename(deserialize = "channelTitle"))]
    pub channel_title: String,
    #[serde(rename(deserialize = "channelId"))]
    pub channel_id: String,
    #[serde(rename(deserialize = "resourceId"))]
    pub resource_id: ResourceId

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
                item.snippet.channel_id,
                item.snippet.channel_title,
                item.snippet.title,
                item.snippet.resource_id.video_id,
                item.snippet.published_at
            )).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::playlist_items::{PlaylistItem, PlaylistItems, ResourceId, Snippet};
    use crate::video::Video;

    #[test]
    fn into_videos_works() {
        let items = PlaylistItems {
            items: vec![
                PlaylistItem {
                    snippet: Snippet {
                        published_at: "2021-12-19T19:13:00Z".to_string(),
                        title: "V0".to_string(),
                        channel_title: "C0".to_string(),
                        channel_id: "CID0".to_string(),
                        resource_id: ResourceId {
                            video_id: "I0".to_string()
                        }
                    }
                },
                PlaylistItem {
                    snippet: Snippet {
                        published_at: "2021-12-18T19:13:00Z".to_string(),
                        title: "V1".to_string(),
                        channel_title: "C1".to_string(),
                        channel_id: "CID1".to_string(),
                        resource_id: ResourceId {
                            video_id: "I1".to_string()
                        }
                    }
                }
            ]
        };
        let _videos: Vec<Video> = items.into();
    }
}