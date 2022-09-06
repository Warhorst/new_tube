const TELEGRAM_API_KEY: &'static str = "NEW_TUBE_TELEGRAM_API_KEY";
const ALLOWED_BOT_USER: &'static str = "NEW_TUBE_ALLOWED_BOT_USER";

pub fn get_telegram_api_key() -> String {
    std::env::var(TELEGRAM_API_KEY).expect("the telegram api key should be set")
}

pub fn get_allowed_bot_user() -> String {
    std::env::var(ALLOWED_BOT_USER).expect("the allowed bot user should be set")
}

