use std::collections::HashMap;

use error_generator::error;
use futures::future::join_all;

use crate::api::caller::{APICaller, ApiCallerError};
use crate::api::playlist_items::{PlaylistItem, PlaylistItems};
use crate::api::video_items::{VideoItem, VideoItems};
use crate::date_helper::date_is_new;
use crate::Video;
use crate::video_retriever::VideoRetrieveError::{ItemAmountMissMatch, ItemIdMissMatch};

pub type PlaylistIdWithTimestamp = (String, String);
pub type Result<T> = std::result::Result<T, VideoRetrieveError>;

pub struct VideoRetriever {
    api_caller: APICaller,
}

impl VideoRetriever {
    pub fn new() -> Result<Self> {
        Ok(VideoRetriever {
            api_caller: APICaller::new()?
        })
    }

    pub async fn get_latest_videos_for_playlist(&self, playlist_id: &str) -> Result<Vec<Video>> {
        let playlist_items = self.api_caller.get_playlist_items(playlist_id).await?;
        let video_ids = playlist_items.get_video_ids();
        let video_items = self.api_caller.get_video_items(video_ids).await?;
        Self::merge_playlist_items_and_video_items(playlist_items, video_items)
    }

    pub async fn get_new_videos_for_playlists(&self, playlist_ids_with_timestamp: Vec<PlaylistIdWithTimestamp>) -> Result<Vec<Video>> {
        let playlist_items = self.get_latest_playlist_items_from_id_timestamp_vec(playlist_ids_with_timestamp).await?;
        let video_ids = playlist_items.get_video_ids();
        let video_items = self.api_caller.get_video_items(video_ids).await?;
        Self::merge_playlist_items_and_video_items(playlist_items, video_items)
    }

    async fn get_latest_playlist_items_from_id_timestamp_vec(&self, playlist_ids_with_timestamp: Vec<PlaylistIdWithTimestamp>) -> Result<PlaylistItems> {
        let outputs =  join_all(
            playlist_ids_with_timestamp
                .into_iter()
                .map(|(id, timestamp)| self.get_latest_playlist_items(id.clone(), timestamp.clone()))
        ).await;

        let playlist_items = outputs.into_iter()
            .map(|res| res.unwrap())
            .fold(PlaylistItems::empty(), |item0, item1| item0.merge(item1));
        Ok(playlist_items)
    }

    async fn get_latest_playlist_items(&self, playlist_id: String, last_video_timestamp: String) -> Result<PlaylistItems> {
        let playlist_items = self.api_caller.get_playlist_items(&playlist_id).await?;
        Ok(PlaylistItems {
            items: playlist_items.items
                .into_iter()
                .filter(|item| date_is_new(&last_video_timestamp, &item.content_details.video_published_at))
                .collect()
        })
    }

    fn merge_playlist_items_and_video_items(playlist_items: PlaylistItems, video_items: VideoItems) -> Result<Vec<Video>> {
        Self::validate_item_amounts_match(&playlist_items, &video_items)?;
        let id_playlist_map = Self::playlist_items_to_id_playlist_map(playlist_items);
        let id_video_map = Self::video_items_to_id_video_map(video_items);
        Self::id_playlist_map_and_id_video_map_to_videos(id_playlist_map, id_video_map)
    }

    fn validate_item_amounts_match(playlist_items: &PlaylistItems, video_items: &VideoItems) -> Result<()> {
        match (playlist_items.items.len(), video_items.items.len()) {
            (pl, vl) if pl != vl => Err(ItemAmountMissMatch { num_playlist_items: pl, num_video_items: vl }),
            _ => Ok(())
        }
    }

    fn playlist_items_to_id_playlist_map(playlist_items: PlaylistItems) -> HashMap<String, PlaylistItem> {
        playlist_items.items
            .into_iter()
            .fold(HashMap::new(), |mut map, item| {
                map.insert(item.snippet.resource_id.video_id.clone(), item);
                map
            })
    }

    fn video_items_to_id_video_map(video_items: VideoItems) -> HashMap<String, VideoItem> {
        video_items.items
            .into_iter()
            .fold(HashMap::new(), |mut map, item| {
                map.insert(item.id.clone(), item);
                map
            })
    }

    fn id_playlist_map_and_id_video_map_to_videos(id_playlist_map: HashMap<String, PlaylistItem>, id_video_map: HashMap<String, VideoItem>) -> Result<Vec<Video>> {
        let mut videos = vec![];
        for (id, playlist) in id_playlist_map {
            let video_item = match id_video_map.get(&id) {
                Some(v) => v,
                None => return Err(ItemIdMissMatch(id))
            };
            videos.push(Video::from_playlist_item_and_video_item(&playlist, video_item))
        }
        Ok(videos)
    }
}

#[error]
pub enum VideoRetrieveError {
    #[error(message = "{_0}", impl_from)]
    ApiCallFailed(ApiCallerError),
    #[error(message = "Amount of playlist items and video items don't match. Playlist items: {num_playlist_items}, video items: {num_video_items}")]
    ItemAmountMissMatch {
        num_playlist_items: usize,
        num_video_items: usize,
    },
    #[error(message = "The video id {_0} is part of the playlist items, but not the video items.")]
    ItemIdMissMatch(String),
}