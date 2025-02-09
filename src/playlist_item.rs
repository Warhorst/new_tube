use crate::new_tube_service::yt_dlp::YTDLPItem;

/// Represents the latest item from a YouTube playlist
#[derive(Clone, Debug)]
pub struct PlaylistItem {
    /// ID of the playlist
    pub playlist_id: String,
    /// ID of the latest video in the playlist
    pub video_id: String,
    /// The tile of the video
    pub title: String,
    /// The duration of the video in seconds.
    /// Might be zero if the video is a currently running livestream
    pub duration: f32,
    /// The channel name which uploaded the video
    pub uploader: String,
    /// The previous video id which was uploaded before the latest one.
    /// Required to check if a video was removed from a playlist.
    pub previous_video_id: String
}

impl PlaylistItem {
    pub fn new(yt_dlp_item: YTDLPItem, previous_video_id: String) -> Self {
        PlaylistItem {
            playlist_id: yt_dlp_item.playlist_id,
            video_id: yt_dlp_item.id,
            title: yt_dlp_item.title,
            duration: yt_dlp_item.duration.unwrap_or_default(),
            uploader: yt_dlp_item.channel,
            previous_video_id: previous_video_id,
        }
    }

    /// Create a full url to the video this playlist item represents
    pub fn link(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.video_id)
    }

    /// Return the duration of the video in a properly formatted string
    pub fn formatted_duration(&self) -> String {
        let secs = self.duration as usize;
        let seconds = secs % 60;
        let minutes = (secs / 60) % 60;
        let hours = (secs / 60) / 60;
        format!("{hours}:{minutes}:{seconds}")
    }
}