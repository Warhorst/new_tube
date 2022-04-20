use error_generator::error;
use telegram_bot::{Api, CanSendMessage, MessageChat};

use crate::{NewTubeService, Video};
use crate::new_tube_service::NewTubeServiceError;

pub type Result<T> = std::result::Result<T, VideoFetcherError>;

/// Fetches new videos and sends the results to telegram.
pub struct VideoFetcher {
    telegram_api: Api,
    chat: MessageChat,
    new_tube_service: NewTubeService,
}

impl VideoFetcher {
    pub fn new(
        telegram_api: Api,
        chat: MessageChat,
    ) -> Result<Self> {
        Ok(VideoFetcher {
            telegram_api,
            chat,
            new_tube_service: NewTubeService::new()?,
        })
    }

    /// Fetch new videos and send the result to the target telegram channel.
    /// If any error occurs, try to send the problem to the channel.
    pub async fn fetch_and_send_new_videos(&self) {
        let res = self.try_fetch_and_send().await;

        if let Err(e) = res {
            self.send_error_message(e).await;
        }
    }

    async fn try_fetch_and_send(&self) -> Result<()> {
        println!("Fetching new videos");
        let new_videos = self.new_tube_service.get_new_videos_and_update_database().await?;

        match new_videos.len() {
            0 => println!("Nothing found"),
            len => {
                println!("Found {len} new videos");
                for v in new_videos {
                    self.send_message_for_video(v).await?;
                }
            }
        }

        Ok(())
    }

    async fn send_message_for_video(&self, video: Video) -> Result<()> {
        self.telegram_api.send(self.chat.text(Self::video_to_telegram_message(video))).await?;
        Ok(())
    }

    fn video_to_telegram_message(video: Video) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            video.channel_name,
            video.name,
            video.formatted_duration(),
            video.link()
        )
    }

    async fn send_error_message(&self, error: VideoFetcherError) {
        if let Err(e) = self.telegram_api.send(self.chat.text(format!("{error}"))).await {
            print!("Failed to send error '{error}' due to other error: {e}");
        }
    }
}

#[error(impl_from)]
pub enum VideoFetcherError {
    #[error(message = "Failed to retrieve new videos. {_0}")]
    NewTubeServiceError(NewTubeServiceError),
    #[error(message = "Failed to send telegram message")]
    SendMessageFailed(telegram_bot::Error),
}