use chrono::{DateTime, Local};
use serde::{Serialize, Serializer};

#[derive(Debug, Serialize)]
pub struct Video {
    pub playlist_id: String,
    pub channel_name: String,
    pub name: String,
    pub id: String,
    #[serde(serialize_with = "serialize_date_time")]
    pub release_date: DateTime<Local>,
}

impl Video {
    /// Create a Video from the given parameters. The release_date must be a valid
    /// RFC 3339 date string (see https://www.ietf.org/rfc/rfc3339.txt).
    /// The release date is in UTC and will be converted to Local.
    pub fn new(
        playlist_id: String,
        channel_name: String,
        name: String,
        id: String,
        release_date: String,
    ) -> Self {
        let utc_release_date = DateTime::parse_from_rfc3339(&release_date).unwrap();

        Video {
            playlist_id,
            channel_name,
            name,
            id,
            release_date: DateTime::<Local>::from(utc_release_date),
        }
    }

    /// Check if the given date is lower than the date of this video. The date string must be a valid
    /// RFC 3339 date string.
    pub fn is_new(&self, last_video_release: &str) -> bool {
        let last_video_release_utc = DateTime::parse_from_rfc3339(last_video_release).unwrap();
        let last_video_release_local = DateTime::<Local>::from(last_video_release_utc);
        last_video_release_local < self.release_date
    }

    pub fn link(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }

    pub fn formatted_release_date(&self) -> String {
        self.release_date.format("%d.%m.%Y %H:%M").to_string()
    }

    pub fn rfc3339_release_date(&self) -> String {
        self.release_date.to_rfc3339()
    }
}

fn serialize_date_time<S>(date_time: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(&date_time.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use crate::video::Video;

    #[test]
    fn create_video_works() {
        Video::new(
            "PlaylistID".to_string(),
            "Channel".to_string(),
            "Video".to_string(),
            "69NICE420".to_string(),
            "2021-12-19T19:13:00Z".to_string(),
        );
    }
}