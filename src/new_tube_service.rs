use std::collections::HashMap;

use error_generator::error;

use crate::{compare_video_releases, Database, Playlist, Video, VideoRetriever};
use crate::new_tube_service::NewTubeServiceError::PlaylistHasNoVideos;

/// Central service of new_tube. Provides methods to add playlists
/// and fetch new videos.
pub struct NewTubeService {
    database: Database,
    video_retriever: VideoRetriever
}

pub type Result<T> = std::result::Result<T, NewTubeServiceError>;

impl NewTubeService {
    pub fn new() -> Result<Self> {
        Ok(NewTubeService {
            database: Database::open()?,
            video_retriever: VideoRetriever::new()?
        })
    }

    pub async fn add_playlist(&self, id: &str) -> Result<()> {
        let latest_videos = self.video_retriever.get_latest_videos_for_playlist(id).await?;
        let playlist = match latest_videos.iter().max_by(|v0, v1| compare_video_releases(v0, v1)) {
            Some(video) => Playlist::from((id, video)),
            None => return Err(PlaylistHasNoVideos)
        };

        self.database.add_playlist(playlist)?;
        Ok(())
    }

    pub async fn get_new_videos_and_update_database(&self) -> Result<Vec<Video>> {
        let playlist_ids_with_timestamp = self.get_playlist_ids_with_timestamp_from_db()?;
        let videos = self.video_retriever.get_new_videos_for_playlists(playlist_ids_with_timestamp).await?;
        self.update_playlists_in_db(&videos)?;
        Ok(videos)
    }

    fn get_playlist_ids_with_timestamp_from_db(&self) -> Result<Vec<(String, String)>> {
        let playlist_ids_with_timestamps = self.database.get_playlists()?
            .into_iter()
            .map(|p| (p.id.clone(), p.last_video_release.clone()))
            .collect();
        Ok(playlist_ids_with_timestamps)
    }

    fn update_playlists_in_db(&self, videos: &Vec<Video>) -> Result<()> {
        let playlist_id_video_map = videos.into_iter().fold(HashMap::<String, Vec<&Video>>::new(), |mut acc, video| {
            let entry = acc.entry(video.playlist_id.clone()).or_insert(vec![]);
            entry.push(video);
            acc
        });

        for videos in playlist_id_video_map.values() {
            let latest_video = videos.into_iter().max_by(|v0, v1| compare_video_releases(v0, v1)).unwrap();
            self.database.update_playlist(latest_video)?;
        }

        Ok(())
    }
}

#[error]
pub enum NewTubeServiceError {
    #[error(message = "{_0}", impl_from)]
    DatabaseAccessFailed(crate::db::DBError),
    #[error(message = "{_0}", impl_from)]
    VideoRetrieveFailed(crate::video_retriever::VideoRetrieveError),
    #[error(message = "The playlist for the given id has no videos.")]
    PlaylistHasNoVideos,
}