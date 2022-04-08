use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

use telegram_bot::{Api, MessageChat};

use crate::telegram::video_fetch_worker::VideoFetchWorker;

/// Manages the background worker which fetches new videos
pub struct BackgroundVideoFetcher {
    worker_thread_sender: Option<Sender<()>>
}

impl BackgroundVideoFetcher {
    pub fn new() -> Self {
        BackgroundVideoFetcher {
            worker_thread_sender: None
        }
    }

    pub fn start(&mut self, telegram_api: Api, chat: MessageChat) {
        if self.worker_already_started() {
            self.stop()
        }

        self.start_worker(telegram_api, chat)
    }

    fn worker_already_started(&self) -> bool {
        self.worker_thread_sender.is_some()
    }

    fn start_worker(&mut self, telegram_api: Api, chat: MessageChat) {
        let (sender, receiver) = mpsc::channel();
        self.worker_thread_sender = Some(sender);

        thread::spawn(move || {
            let mut runtime = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
            runtime.block_on(VideoFetchWorker::new(telegram_api, chat, receiver).unwrap().periodically_fetch_new_videos())
        });
    }

    pub fn stop(&mut self) {
        match &self.worker_thread_sender {
            Some(sender) => {
                sender.send(()).expect("Could not send stop message to worker thread");
                self.worker_thread_sender = None;
                println!("Worker successfully stopped");
            },
            None => println!("Worker was not started")
        }
    }
}

