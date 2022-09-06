use frankenstein::{Api, SendMessageParams, TelegramApi};

use crate::{Item, NewTubeService};
use crate::new_tube_service::NewTubeServiceError;

/// Fetches new videos and sends the results to telegram.
pub struct VideoFetcher {
    api: Api,
    new_tube_service: NewTubeService
}

impl VideoFetcher {
    pub fn new(api: Api) -> Result<Self, NewTubeServiceError> {
        Ok(VideoFetcher {
            api,
            new_tube_service: NewTubeService::new()?
        })
    }

    pub fn fetch_and_send_new_videos(&self, chat_id: i64) {
        println!("Fetching new videos");
        let new_videos = match self.new_tube_service.get_new_videos_and_update_database() {
            Ok(vs) => vs,
            Err(error) => {
                println!("{error}");
                return
            }
        };

        match new_videos.len() {
            0 => println!("Nothing found"),
            len => {
                println!("Found {len} new videos");
                new_videos.into_iter().for_each(|v| self.send_message(chat_id, v))
            }
        }
    }

    fn send_message(&self, chat_id: i64, item: Item) {
        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(Self::item_to_telegram_message(&item))
            .build();

        if let Err(error) = self.api.send_message(&params) {
            println!("Failed to send message for item {:?}. Error: {}", item, error)
        }
    }

    fn item_to_telegram_message(item: &Item) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            item.uploader,
            item.title,
            item.formatted_duration(),
            item.link()
        )
    }
}