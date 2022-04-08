use error_generator::error;
use futures::StreamExt;
use telegram_bot::{Api, Message, MessageKind, Update, UpdateKind};

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

        while let Some(update_result) = stream.next().await {
            if let Some((text, message)) = self.get_text_and_message_from_update(update_result?) {
                match text.as_str() {
                    "/start" => {
                        self.video_fetch_worker.start(api.clone(), message.chat)
                    },
                    "/stop" => {
                        self.video_fetch_worker.stop()
                    },
                    _ => println!("{text}")
                };
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
}

#[error(impl_from)]
pub enum BotError {
    #[error(message = "Error while retrieving new videos: {_0}")]
    RetrieveNewVideosFailed(crate::new_tube_service::NewTubeServiceError),
    #[error(message = "Telegram API key could not be retrieved: {_0}")]
    TelegramApiKeyNotRetrieved(std::env::VarError),
    #[error(message = "Error while calling the telegram API: {_0}")]
    TelegramApiError(telegram_bot::Error),
}

