use futures::executor::block_on;
use teloxide::prelude2::*;
use teloxide::Bot;
use crate::Database;

pub struct NTBot {
    database: Database,
}

impl NTBot {
    pub fn new() -> Self {
        NTBot { database: Database::open().unwrap() }
    }

    pub fn run(self) {
        block_on(self.start())
    }

    pub async fn start(&self) {
        let bot = Bot::from_env().auto_send();

        teloxide::repls2::repl(bot, |message: Message, bot: AutoSend<Bot>| async move {
            respond(())
        });
    }
}

