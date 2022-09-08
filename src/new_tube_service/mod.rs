use error_generator::error;

use crate::Item;
use crate::new_tube_service::database::{Database, DBError};
use crate::new_tube_service::NewTubeServiceError::PlaylistHasNoVideos;
use crate::new_tube_service::yt_dlp::{Error, Items, YTDLPCaller};

pub mod database;
pub mod yt_dlp;

type Result<T> = std::result::Result<T, NewTubeServiceError>;

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
        let latest_items = YTDLPCaller::retrieve_latest_items(id)?;
        let latest_item = match latest_items.get(0) {
            Some(item) => item,
            None => return Err(PlaylistHasNoVideos)
        };

        self.database.add_item(latest_item)?;
        Ok(())
    }

    pub fn get_new_videos_and_update_database(&self) -> Result<Items> {
        let last_items = self.database.get_items()?;
        let mut new_items = vec![];

        for last in last_items {
            new_items.extend(YTDLPCaller::retrieve_latest_items(&last.playlist_id)?
                .into_iter()
                .take_while(|item| item.video_id != last.video_id));
        }

        self.save_new_videos(&new_items)?;

        Ok(new_items)
    }

    pub fn get_last_items(&self) -> Result<Items> {
        Ok(self.database.get_items()?.into_iter().collect())
    }

    pub fn get_new_videos(&self, last: &Item) -> Result<Vec<Item>> {
        Ok(YTDLPCaller::retrieve_latest_items(&last.playlist_id)?
            .into_iter()
            .take_while(|item| item.video_id != last.video_id)
            .collect())
    }

    /// Add all new items to the database.
    ///
    /// Items from yt_dlp are already ordered. This means newer items are at the top
    /// while new items are at the bottom of the list. Therefore, when adding all items to the database,
    /// the list gets reversed, so the older items are added first. This prevents the database from holding
    /// the older items rather than the new ones.
    ///
    /// The items have an upload date, but its precision is 'day', so this is the best solution if a
    /// channel uploads multiple times a day.
    pub fn save_new_videos(&self, new_items: &Items) -> Result<()> {
        for item in new_items.iter().rev() {
            self.database.add_item(&item)?
        }

        Ok(())
    }
}

#[error]
pub enum NewTubeServiceError {
    #[error(message = "{_0}", impl_from)]
    DatabaseAccessFailed(DBError),
    #[error(message = "{_0}", impl_from)]
    YTDLPError(Error),
    #[error(message = "The playlist for the given id has no videos.")]
    PlaylistHasNoVideos,
}