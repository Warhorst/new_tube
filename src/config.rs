use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    /// the time in minutes the bot will wait before fetching again
    pub bot_fetch_schedule: u32
}