use serde::Serialize;

use crate::api::playlist_items::PlaylistItem;
use crate::api::video_items::VideoItem;
use crate::date_helper::string_to_local_time_date;
use crate::duration_formatter::format_duration;

#[derive(Debug, Serialize)]
pub struct Video {
    pub playlist_id: String,
    pub channel_name: String,
    pub name: String,
    pub id: String,
    pub release_date: String,
    pub duration: String
}

impl Video {
    pub fn new(
        playlist_id: String,
        channel_name: String,
        name: String,
        id: String,
        release_date: String,
        duration: String
    ) -> Self {
        Video {
            playlist_id,
            channel_name,
            name,
            id,
            release_date,
            duration
        }
    }

    pub fn from_playlist_item_and_video_item(playlist_item: &PlaylistItem, video_item: &VideoItem) -> Self {
        Self::new(
            playlist_item.snippet.playlist_id.clone(),
            playlist_item.snippet.channel_title.clone(),
            playlist_item.snippet.title.clone(),
            playlist_item.snippet.resource_id.video_id.clone(),
            playlist_item.content_details.video_published_at.clone(),
            video_item.content_details.duration.clone()
        )
    }

    pub fn link(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }

    pub fn formatted_release_date(&self) -> String {
        string_to_local_time_date(&self.release_date).format("%d.%m.%Y %H:%M").to_string()
    }

    pub fn formatted_duration(&self) -> String {
        format_duration(self.duration.clone())
    }
}