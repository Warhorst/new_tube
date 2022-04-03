use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use telegram_bot::{Api, CanSendMessage, MessageChat};

use crate::{NewTubeService, Video};

pub struct VideoFetchWorker {
    worker_thread_sender: Option<Sender<()>>
}

impl VideoFetchWorker {
    pub fn new() -> Self {
        VideoFetchWorker {
            worker_thread_sender: None
        }
    }

    pub fn start(&mut self, telegram_api: Api, chat: MessageChat) {
        let (sender, receiver) = mpsc::channel();
        self.worker_thread_sender = Some(sender);

        thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(Self::periodically_fetch_new_videos(telegram_api, receiver, chat))
        });
    }

    async fn periodically_fetch_new_videos(telegram_api: Api, receiver: Receiver<()>, chat: MessageChat) {
        println!("Starting worker thread");

        let new_tube_service = match NewTubeService::new() {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to create worker due to error: {e}");
                return;
            }
        };
        let mut i = 600;

        loop {
            if Self::worker_must_be_stopped(&receiver) {
                break;
            }

            if Self::it_is_time_to_fetch_videos(i) {
                i = 0;
                Self::fetch_and_send_new_videos(&telegram_api, &chat, &new_tube_service).await;
            }

            i += 1;
            Self::sleep_500_milliseconds()
        }
    }

    fn worker_must_be_stopped(receiver: &Receiver<()>) -> bool {
        match receiver.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => true,
            Err(TryRecvError::Empty) => false
        }
    }

    /// We wait 500ms every loop, and every 5 minutes videos shall be fetched.
    /// 600 loops are equivalent to waiting 600 times 0.5 seconds, which is
    /// 5 minutes in total.
    /// This is a very dumb way of scheduling and should be changed in the future.
    fn it_is_time_to_fetch_videos(i: usize) -> bool {
        i == 600
    }

    async fn fetch_and_send_new_videos(telegram_api: &Api, chat: &MessageChat, new_tube_service: &NewTubeService) {
        println!("Fetching new videos");
        let new_videos = new_tube_service.get_new_videos_and_update_database().await.unwrap();
        match new_videos.len() {
            0 => println!("Nothing found"),
            len => {
                println!("Found {len} new videos");
                for v in new_videos {
                    Self::send_message_for_video(telegram_api, chat, v).await
                }
            }
        }
    }

    async fn send_message_for_video(api: &Api, chat: &MessageChat, video: Video) {
        api.send(chat.text(Self::video_to_telegram_message(video))).await.unwrap();
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

    pub fn stop(&mut self) {
        match &self.worker_thread_sender {
            Some(sender) => {
                sender.send(()).unwrap();
                self.worker_thread_sender = None
            },
            None => println!("Worker was not started")
        }
    }
}

