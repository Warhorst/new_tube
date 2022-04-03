use std::env::VarError;

const YOUTUBE_API_KEY: &'static str = "NEW_TUBE_YOUTUBE_API_KEY";
const TELEGRAM_API_KEY: &'static str = "NEW_TUBE_TELEGRAM_API_KEY";
const ALLOWED_BOT_USER: &'static str = "NEW_TUBE_ALLOWED_BOT_USER";

pub fn get_youtube_api_key() -> Result<String, VarError> {
    std::env::var(YOUTUBE_API_KEY)
}

pub fn get_telegram_api_key() -> Result<String, VarError> {
    std::env::var(TELEGRAM_API_KEY)
}

pub fn get_allowed_bot_user() -> Result<String, VarError> {
    std::env::var(ALLOWED_BOT_USER)
}

