use std::thread;
use std::time::Duration;

use clokwerk::{Scheduler, TimeUnits};
use error_generator::error;
use frankenstein::{AllowedUpdate, Api, GetUpdatesParams, Message, SendMessageParams, TelegramApi, UpdateContent};

use crate::config::Config;
use crate::environment::{get_allowed_bot_user, get_default_telegram_channel_id, get_telegram_api_key};
use crate::new_tube_service::NewTubeServiceError;
use crate::new_tube_service::yt_dlp::Item;
use crate::telegram_bot::video_fetcher::VideoFetcher;

mod video_fetcher;

pub struct Bot;

impl Bot {
    pub fn run(config: Config) -> Result<(), BotError> {
        let mut scheduler = Scheduler::new();
        let api = Api::new(&get_telegram_api_key());
        let chat_id = get_default_telegram_channel_id();
        let fetcher = VideoFetcher::new()?;

        scheduler.every(10.seconds()).run(Self::read_updates(api.clone(), chat_id));
        scheduler.every(config.bot_fetch_schedule.minutes()).run(Self::fetch_videos(api.clone(), chat_id, fetcher));
        Self::send_message(&api, chat_id, "Started");
        println!("Bot started");

        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(500));
        }
    }

    fn fetch_videos(api: Api, chat_id: i64, video_fetcher: VideoFetcher) -> impl FnMut() {
        move || {
            match video_fetcher.fetch_new_videos() {
                Ok(new_videos) => new_videos.into_iter()
                    .map(Self::item_to_telegram_message)
                    .for_each(|m| Self::send_message(&api, chat_id, m)),
                Err(err) => println!("An error occurred while fetching new videos: {}", err)
            }
        }
    }

    /// Get the latest updates to the bot and process them.
    ///
    /// The updates are returned by telegrams getUpdates method (https://core.telegram.org/bots/api#getupdates).
    /// A last update id and a message filter is provided. The last update id is important, as
    /// an update is only considered processed if an id larger than its own was provided as the 'offset'
    /// parameter. Therefore, the last update id is stored and provided as a parameter.
    fn read_updates(api: Api, chat_id: i64) -> impl FnMut() {
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
                            Self::process_update_message(&api, chat_id, message)
                        }
                    }
                }
                Err(error) => {
                    println!("Error while fetching telegram updates: {error}")
                }
            }
        }
    }

    fn process_update_message(api: &Api, chat_id: i64, message: Message) {
        if !Self::sender_is_valid(&message) {
            return;
        }

        if let Some(ref text) = message.text {
            match text.as_str() {
                // simply check if the bot is still running
                "/health" => {
                    Self::send_message(api, chat_id, "I am alive")
                }
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

    fn send_message(api: &Api, chat_id: i64, message: impl ToString) {
        let params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(message.to_string())
            .build();

        if let Err(err) = api.send_message(&params) {
            println!("failed to send message due to error: {}", err)
        }
    }

    fn item_to_telegram_message(item: Item) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            item.uploader,
            item.title,
            item.formatted_duration(),
            item.link()
        )
    }
}

#[error]
pub enum BotError {
    #[error(message = "{_0}", impl_from)]
    NewTubeService(NewTubeServiceError)
}