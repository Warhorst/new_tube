use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use telegram_bot::{Api, CanSendMessage, MessageChat};

use crate::{NewTubeService, Video};
use crate::new_tube_service::NewTubeServiceError;

/// Worker which periodically fetches new videos
/// and sends them to telegram. Might be stopped by its master.
pub struct VideoFetchWorker {
    telegram_api: Api,
    chat: MessageChat,
    receiver: Receiver<()>,
    new_tube_service: NewTubeService,
    current_iteration: usize,
}

impl VideoFetchWorker {
    pub fn new(
        telegram_api: Api,
        chat: MessageChat,
        receiver: Receiver<()>,
    ) -> Result<Self, NewTubeServiceError> {
        Ok(VideoFetchWorker {
            telegram_api,
            chat,
            receiver,
            new_tube_service: NewTubeService::new()?,
            current_iteration: 600
        })
    }

    pub async fn periodically_fetch_new_videos(&mut self) {
        println!("Starting worker thread");
        loop {
            if self.worker_must_be_stopped() {
                break;
            }

            if self.it_is_time_to_fetch_videos() {
                self.current_iteration = 0;
                self.fetch_and_send_new_videos().await;
            }

            self.current_iteration += 1;
            Self::sleep_500_milliseconds()
        }
    }

    fn worker_must_be_stopped(&self) -> bool {
        match self.receiver.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => true,
            Err(TryRecvError::Empty) => false
        }
    }

    /// We wait 500ms every loop, and every 5 minutes videos shall be fetched.
    /// 600 loops are equivalent to waiting 600 times 0.5 seconds, which is
    /// 5 minutes in total.
    /// This is a very dumb way of scheduling and should be changed in the future.
    fn it_is_time_to_fetch_videos(&self) -> bool {
        self.current_iteration == 600
    }

    async fn fetch_and_send_new_videos(&self) {
        println!("Fetching new videos");
        let new_videos = self.new_tube_service.get_new_videos_and_update_database().await.unwrap();
        match new_videos.len() {
            0 => println!("Nothing found"),
            len => {
                println!("Found {len} new videos");
                for v in new_videos {
                    self.send_message_for_video(v).await
                }
            }
        }
    }

    async fn send_message_for_video(&self, video: Video) {
        self.telegram_api.send(self.chat.text(Self::video_to_telegram_message(video))).await.unwrap();
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

    fn sleep_500_milliseconds() {
        thread::sleep(Duration::from_millis(500))
    }
}