use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError, TryRecvError};
use std::thread;
use std::time::Duration;

use clokwerk::{Scheduler, TimeUnits};
use error_generator::error;
use telegram_bot::{Api, MessageChat};

use crate::tg_bot::background_video_fetcher::BackgroundVideoFetcherError::FailedToStopBecauseWorkerNotStarted;
use crate::tg_bot::video_fetch_worker::{VideoFetcher, VideoFetcherError};

pub type Result<T> = std::result::Result<T, BackgroundVideoFetcherError>;

/// Manages the background worker which fetches new videos
pub struct BackgroundVideoFetcher {
    worker_thread_sender: Option<Sender<()>>,
}

impl BackgroundVideoFetcher {
    pub fn new() -> Self {
        BackgroundVideoFetcher {
            worker_thread_sender: None
        }
    }

    pub fn start(&mut self, telegram_api: Api, chat: MessageChat) -> Result<()> {
        if self.worker_already_started() {
            self.stop()?
        }

        self.start_worker(telegram_api, chat)?;
        Ok(())
    }

    fn worker_already_started(&self) -> bool {
        self.worker_thread_sender.is_some()
    }

    fn start_worker(&mut self, telegram_api: Api, chat: MessageChat) -> Result<()> {
        let (sender, receiver) = mpsc::channel();
        self.worker_thread_sender = Some(sender);
        let mut runtime = tokio::runtime::Runtime::new()?;
        let video_fetcher = VideoFetcher::new(telegram_api, chat)?;

        thread::spawn(move || {
            let mut scheduler = Scheduler::new();

            scheduler.every(5.minutes()).run(move || {
                runtime.block_on(video_fetcher.fetch_and_send_new_videos());
            });

            loop {
                scheduler.run_pending();
                thread::sleep(Duration::from_millis(500));
                if Self::worker_must_be_stopped(&receiver) { break; }
            }
        });

        Ok(())
    }

    fn worker_must_be_stopped(receiver: &Receiver<()>) -> bool {
        match receiver.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => true,
            Err(TryRecvError::Empty) => false
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        match &self.worker_thread_sender {
            Some(sender) => {
                sender.send(())?;
                self.worker_thread_sender = None;
                Ok(())
            }
            None => Err(FailedToStopBecauseWorkerNotStarted)
        }
    }
}

#[error]
pub enum BackgroundVideoFetcherError {
    #[error(message = "Could not start new runtime for worker: {_0}", impl_from)]
    AsyncRuntimeFailedToStart(std::io::Error),
    #[error(message = "Could not create video fetcher: {_0}", impl_from)]
    FailedToCreateVideoFetcher(VideoFetcherError),
    #[error(message = "Could not send stop message to worker: {_0}", impl_from)]
    FailedToSendStopMessage(SendError<()>),
    #[error(message = "Could not stop worker because it was not started")]
    FailedToStopBecauseWorkerNotStarted
}

