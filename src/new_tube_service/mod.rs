use error_generator::error;

use crate::new_tube_service::database::{DBError, Database};
use crate::new_tube_service::yt_dlp::{retrieve_latest_items, Error, YTDLPResponse};
use crate::playlist_item::PlaylistItem;

pub mod database;
pub mod yt_dlp;

pub type Result<T> = std::result::Result<T, NewTubeServiceError>;

pub struct NewTubeService {
    database: Database,
}

impl NewTubeService {
    pub fn new() -> Result<Self> {
        Ok(NewTubeService {
            database: Database::open()?
        })
    }

    pub fn add_playlist(&self, id: &str) -> Result<()> {
        let response = retrieve_latest_items(id)?;
        let latest = response.latest_item;
        let previous = response.previous_item;

        let item = PlaylistItem {
            playlist_id: latest.playlist_id,
            video_id: latest.id,
            title: latest.title,
            duration: latest.duration.unwrap_or_default(),
            uploader: latest.channel,
            previous_video_id: previous.id,
        };

        self.database.add_item(&item)?;
        Ok(())
    }

    pub fn get_new_videos_and_update_database(&self) -> Result<Vec<PlaylistItem>> {
        let last_items = self.database.query_all_items()?;
        let mut new_videos = vec![];

        for last in last_items {
            let new = self.get_new_video(&last)?;

            match new {
                // A new video which must be saved and returned to the user
                NewVideo::ReallyNew(video) => {
                    self.database.add_item(&video)?;
                    new_videos.push(video);
                }
                // A video which replaces a now removed one.
                // This needs only to be saved
                NewVideo::OldVideoNowLatest(video) => {
                    self.database.add_item(&video)?;
                }
                // Nothing new, so nothing to do here
                NewVideo::SameAsBefore => {}
            }
        }

        Ok(new_videos)

    }

    fn get_new_video(&self, last: &PlaylistItem) -> Result<NewVideo> {
        let YTDLPResponse {latest_item, previous_item} = retrieve_latest_items(&last.playlist_id)?;
        let latest = PlaylistItem::new(latest_item, previous_item.id.clone());

        if latest.video_id == last.video_id {
            // The latest video did not change, so no new video here
            Ok(NewVideo::SameAsBefore)
        } else if latest.video_id == last.previous_video_id {
            // The latest video of the playlist is now the previous latest from the database.
            // This means the current latest video stored in the database was removed from the
            // playlist for any reason. In this case, just return what yt_dlp currently returned as
            // the latest one, as this will overwrite the now invalid entry in the db.
            Ok(NewVideo::OldVideoNowLatest(latest))
        } else {
            // The video is not the same as the current one and also not a previous one.
            // It is really new
            Ok(NewVideo::ReallyNew(latest))
        }
    }

    pub fn replace(&self, old_id: &str, new_id: &str) -> Result<()> {
        self.delete(old_id)?;
        self.add_playlist(new_id)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        Ok(self.database.delete(id)?)
    }
}

/// The different states a "new video" returned by the service can be in
pub enum NewVideo {
    /// The video really is new and should be broadcast to the user
    ReallyNew(PlaylistItem),
    /// A video was removed from a playlist, leaving this video as the new latest one.
    /// This should not be broadcast to the user, as they already know it
    OldVideoNowLatest(PlaylistItem),
    /// No new video. The service returned the same video that is already stored
    SameAsBefore
}

#[error]
pub enum NewTubeServiceError {
    #[error(message = "{_0}", impl_from)]
    DatabaseAccessFailed(DBError),
    #[error(message = "{_0}", impl_from)]
    YTDLPError(Error),
}