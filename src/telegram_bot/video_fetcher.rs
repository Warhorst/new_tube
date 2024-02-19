use crate::{Item, NewTubeService};
use crate::new_tube_service::NewTubeServiceError;

/// Fetches new videos and sends the results to telegram.
pub struct VideoFetcher {
    new_tube_service: NewTubeService
}

impl VideoFetcher {
    pub fn new() -> Result<Self, NewTubeServiceError> {
        Ok(VideoFetcher {
            new_tube_service: NewTubeService::new()?
        })
    }

    pub fn fetch_new_videos(&self) -> Result<Vec<Item>, NewTubeServiceError> {
        println!("Fetching new videos");

        let last_items = self.new_tube_service.get_last_items()?;

        let mut new_videos = vec![];

        for item in last_items {
            let new = self.new_tube_service.get_new_videos(&item)?;
            self.new_tube_service.save_new_videos(&new)?;
            new_videos.extend(new.into_iter());
        }

        match new_videos.len() {
            0 => println!("Nothing found"),
            len => println!("Found {len} new videos"),
        }

        Ok(new_videos)
    }
}