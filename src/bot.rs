use std::thread;
use std::time::Duration;

use error_generator::error;
use teloxide::prelude2::*;
use teloxide::RequestError;

use crate::{Video, VideoService};

type Result<T> = std::result::Result<T, BotError>;

pub struct Bot {
    video_service: VideoService,
}

impl Bot {
    const NEWTUBE_TARGET_TELEGRAM_CHANNEL: &'static str = "NEWTUBE_TARGET_TELEGRAM_CHANNEL";

    pub fn new() -> Result<Self> {
        Ok(Bot {
            video_service: VideoService::new()?
        })
    }

    pub async fn run(self) -> Result<()> {
        let bot = teloxide::Bot::from_env().auto_send();
        let id = Self::get_target_channel_id();

        loop {
            println!("Searching for new videos...");
            let new_videos = self.video_service.get_new_videos_and_update_database().await?;

            if new_videos.is_empty() {
                println!("Nothing found");
            }

            for v in new_videos {
                bot.send_message(id, Self::video_to_telegram_message(v)).await?;
            }

            thread::sleep(Duration::from_secs(5 * 60));
        }
    }

    fn get_target_channel_id() -> i64 {
        std::env::var(Self::NEWTUBE_TARGET_TELEGRAM_CHANNEL).unwrap().parse::<i64>().unwrap()
    }

    fn video_to_telegram_message(video: Video) -> String {
        format!(
            "Channel: {}\nVideo: {}\nDuration: {}\nLink: {}",
            video.channel_name,
            video.name,
            video.formatted_duration(),
            video.link()
        )
    }
}

#[error(impl_from)]
pub enum BotError {
    #[error(message = "Error while retrieving new videos: {_0}")]
    RetrieveNewVideosFailed(crate::video_service::VideoServiceError),
    #[error(message = "Error while calling telegram: {_0}")]
    TelegramCallFailed(RequestError)
}

