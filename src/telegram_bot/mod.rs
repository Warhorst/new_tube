use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use clokwerk::{Scheduler, TimeUnits};
use error_generator::error;
use frankenstein::{Api, SendMessageParams, TelegramApi};

use crate::config::Config;
use crate::environment::{get_default_telegram_channel_id, get_telegram_api_key};
use crate::new_tube_service::NewTubeServiceError;
use crate::telegram_bot::video_fetcher::VideoFetcher;

mod video_fetcher;

pub struct Bot;

impl Bot {
    pub fn run(config: Config) -> Result<(), BotError> {
        let mut scheduler = Scheduler::new();
        let api = Api::new(&get_telegram_api_key());
        let target_chat_id = Arc::new(Mutex::new(Some(get_default_telegram_channel_id())));
        let fetcher = VideoFetcher::new(api.clone())?;

        scheduler.every(config.bot_fetch_schedule.minutes()).run(Self::fetch_videos_if_enabled(target_chat_id.clone(), fetcher));
        Self::send_verification_message_to_channel(&api, &target_chat_id);
        println!("Bot started");

        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(500));
        }
    }

    fn fetch_videos_if_enabled(target_chat_id: Arc<Mutex<Option<i64>>>, video_fetcher: VideoFetcher) -> impl FnMut() {
        move || {
            if let Some(chat_id) = *target_chat_id.lock().unwrap() {
                video_fetcher.fetch_and_send_new_videos(chat_id)
            }
        }
    }

    /// Use the api and the channel id to send a message to the target channel.
    ///
    /// This method is only used to verify the set channel id is correct. If I get a message, everything
    /// is fine.
    fn send_verification_message_to_channel(api: &Api, target_chat_id: &Arc<Mutex<Option<i64>>>) {
        let id = target_chat_id.lock().unwrap().unwrap();
        let params = SendMessageParams::builder().chat_id(id).text("Started").build();
        api.send_message(&params).expect("failed to send verification message");
    }
}

#[error]
pub enum BotError {
    #[error(message = "{_0}", impl_from)]
    NewTubeService(NewTubeServiceError)
}