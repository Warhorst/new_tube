use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use clokwerk::{Scheduler, TimeUnits};
use error_generator::error;
use frankenstein::{AllowedUpdate, Api, GetUpdatesParams, Message, TelegramApi, UpdateContent};

use crate::environment::{get_allowed_bot_user, get_telegram_api_key};
use crate::new_tube_service::NewTubeServiceError;
use crate::telegram_bot::video_fetcher::VideoFetcher;

mod video_fetcher;

pub struct Bot;

impl Bot {
    pub fn run() -> Result<(), BotError> {
        let mut scheduler = Scheduler::new();
        let api = Api::new(&get_telegram_api_key());
        let target_chat_id = Arc::new(Mutex::new(None));
        let fetcher = VideoFetcher::new(api.clone())?;

        scheduler.every(10.seconds()).run(Self::read_updates(target_chat_id.clone(), api));
        scheduler.every(5.minutes()).run(Self::fetch_videos_if_enabled(target_chat_id.clone(), fetcher));

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

    /// Get the latest updates to the bot and process them.
    ///
    /// The updates are returned by telegrams getUpdates method (https://core.telegram.org/bots/api#getupdates).
    /// A last update id and a message filter is provided. The last update id is important, as
    /// an update is only considered processed if an id larger than its own was provided as the 'offset'
    /// parameter. Therefore, the last update id is stored and provided as a parameter.
    fn read_updates(target_chat: Arc<Mutex<Option<i64>>>, api: Api) -> impl FnMut() {
        let mut last_update_id = 0;

        move || {
            let update_params = GetUpdatesParams::builder()
                .offset(last_update_id + 1)
                .allowed_updates(vec![AllowedUpdate::Message])
                .build();

            match api.get_updates(&update_params) {
                Ok(response) => {
                    for update in response.result {
                        last_update_id = update.update_id;

                        if let UpdateContent::Message(message) = update.content {
                            Self::process_update_message(message, &target_chat)
                        }
                    }
                }
                Err(error) => {
                    println!("Error while fetching telegram updates: {error}")
                }
            }
        }
    }

    /// If a target chat id is set, the fetcher should send updates to it. These commands
    /// just set or unset the current chat id.
    fn process_update_message(message: Message, target_chat: &Arc<Mutex<Option<i64>>>) {
        if !Self::sender_is_valid(&message) {
            return;
        }

        if let Some(ref text) = message.text {
            match text.as_str() {
                "/start" => {
                    println!("Started fetching videos");
                    *target_chat.lock().unwrap() = Some(message.chat.id)
                }
                "/stop" => {
                    println!("Stopped fetching videos");
                    *target_chat.lock().unwrap() = None
                },
                _ => ()
            }
        }
    }

    /// The sender must be the allowed bot user and the sender must be a human
    fn sender_is_valid(message: &Message) -> bool {
        let allowed_user = get_allowed_bot_user();

        match message.from {
            Some(ref from) => match from.username {
                Some(ref username) => username == &allowed_user && from.is_bot == false,
                None => false
            },
            None => false
        }
    }
}

#[error]
pub enum BotError {
    #[error(message = "{_0}", impl_from)]
    NewTubeService(NewTubeServiceError)
}