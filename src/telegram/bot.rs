use error_generator::error;
use futures::StreamExt;
use telegram_bot::{Api, Message, MessageKind, Update, UpdateKind};

use environment::get_allowed_bot_user;

use crate::{environment, new};
use crate::environment::get_telegram_api_key;
use crate::telegram::background_video_fetcher::BackgroundVideoFetcher;

type BotResult<T> = Result<T, BotError>;

pub struct Bot {
    video_fetch_worker: BackgroundVideoFetcher
}

impl Bot {
    pub fn new() -> BotResult<Self> {
        Ok(Bot {
            video_fetch_worker: BackgroundVideoFetcher::new()
        })
    }

    pub async fn run(mut self) -> BotResult<()> {
        let api = Api::new(get_telegram_api_key()?);
        let mut stream = api.stream();

        println!("Bot successfully started");

        while let Some(update_result) = stream.next().await {
            if let Some((text, message)) = self.get_text_and_message_from_update(update_result?) {
                if self.message_sender_is_valid_user(&message) {
                    self.process_message(&api, message, text)
                } else {
                    println!("Someone unknown tried to use the bot: {:?}", message);
                }
            }
        }

        Ok(())
    }

    fn get_text_and_message_from_update(&self, update: Update) -> Option<(String, Message)> {
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                return Some((data.clone(), message))
            }
        }

        None
    }

    fn message_sender_is_valid_user(&self, message: &Message) -> bool {
        let allowed_user = get_allowed_bot_user().expect("Failed to get allowed user from environment");

        match message.from.username {
            Some(ref name) => &allowed_user == name && message.from.is_bot == false,
            None => false
        }
    }

    fn process_message(&mut self, api: &Api, message: Message, text: String) {
        match text.as_str() {
            "/start" => match self.video_fetch_worker.start(api.clone(), message.chat) {
                Ok(_) => println!("Background worker successfully started"),
                Err(e) => println!("Failed to start background worker: {e}"),
            },
            "/stop" => match self.video_fetch_worker.stop() {
                Ok(_) => println!("Successfully stopped background worker"),
                Err(e) => println!("Failed to stop background worker: {e}")
            },
            _ => println!("{text}")
        };
    }
}

#[error(impl_from)]
pub enum BotError {
    #[error(message = "Error while retrieving new videos: {_0}")]
    RetrieveNewVideosFailed(new::NewTubeServiceError),
    #[error(message = "Telegram API key could not be retrieved: {_0}")]
    TelegramApiKeyNotRetrieved(std::env::VarError),
    #[error(message = "Error while calling the telegram API: {_0}")]
    TelegramApiError(telegram_bot::Error),
}

