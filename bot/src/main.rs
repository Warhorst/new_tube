use serde::Deserialize;
use std::process::Command;
use std::thread;
use std::time::Duration;
use teloxide::prelude2::*;

#[tokio::main]
async fn main() {
    let bot = Bot::from_env().auto_send();
    teloxide::repls2::repl(bot, |message: Message, bot: AutoSend<Bot>| async move {
        if let Some(user) = message.from() {
            if let Some(name) = &user.username {
                println!("Username: {name}");
            } else {
                println!("Name not set");
            }
        } else {
            println!("User not set")
        }

        // loop {
        //     let videos = get_new_videos();
        //
        //     if !videos.is_empty() {
        //         let m = videos.into_iter().map(Video::to_message).fold(String::new(), |mut acc, item| {
        //             acc = acc + &item;
        //             acc = acc + "\n";
        //             acc
        //         });
        //         bot.send_message(message.chat.id, m).await?;
        //     } else {
        //         println!("Nothing was found")
        //     }
        //
        //     thread::sleep(Duration::from_secs(5 * 60));
        // }

        respond(())
    }).await;
}

fn get_new_videos() -> Vec<Video> {
    let json = get_new_json();
    if !json.is_empty() {
        serde_json::from_str(&json).expect("JSON parse failed")
    } else {
        vec![]
    }
}

fn get_new_json() -> String {
    // TODO: very stupid, fix later
    let result = String::from_utf8(Command::new("cmd").args(["/k", "new-tube", "new_json"]).output().expect("bew_tube new-json failed").stdout).expect("String from byte stream failed");
    result.split(">").collect::<Vec<_>>()[1].to_string()
}

#[derive(Deserialize)]
pub struct Video {
    pub playlist_id: String,
    pub channel_name: String,
    pub name: String,
    pub id: String,
    pub release_date: String,
    pub duration: String
}

impl Video {
    pub fn to_message(self) -> String {
        let mut message = String::new();
        message = message + &self.channel_name + "\n";
        message = message + &self.name + "\n";
        message = message + &format!("https://www.youtube.com/watch?v={}", self.id) + "\n";
        message
    }
}