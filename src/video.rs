use chrono::{DateTime, Local};
use cli_table::row::{Row, ToRow};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Video {
    pub playlist_id: String,
    pub channel_name: String,
    pub name: String,
    pub id: String,
    pub release_date: String,
}

impl Video {
    /// Create a Video from the given parameters. The release_date must be a valid
    /// RFC 3339 date string (see https://www.ietf.org/rfc/rfc3339.txt).
    pub fn new(
        playlist_id: String,
        channel_name: String,
        name: String,
        id: String,
        release_date: String,
    ) -> Self {
        Video {
            playlist_id,
            channel_name,
            name,
            id,
            release_date,
        }
    }

    /// Check if the given date is lower than the date of this video. The date string must be a valid
    /// RFC 3339 date string.
    pub fn is_new(&self, last_video_release: &str) -> bool {
        let last_video_release = DateTime::parse_from_rfc3339(last_video_release).unwrap();
        let video_release = DateTime::parse_from_rfc3339(&self.release_date).unwrap();
        last_video_release < video_release
    }

    fn create_video_link(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }

    fn get_localtime_release_date(&self) -> String {
        let utc_date_time = DateTime::parse_from_rfc3339(&self.release_date).unwrap();
        let local_date_time = DateTime::<Local>::from(utc_date_time);
        local_date_time.format("%d.%m.%Y %H:%M").to_string()
    }
}

impl ToRow<4> for Video {
    fn to_table_row(&self) -> Row<4> {
        Row::from([
            self.channel_name.clone(),
            shorten_video_name(self.name.clone()),
            self.create_video_link(),
            self.get_localtime_release_date()
        ])
    }
}

/// Names can get to long to display in a table cell. Limit to max 45 letters.
/// TODO: fix this in cli_table, not here
fn shorten_video_name(name: String) -> String {
    match name.len() < 45 {
        true => name,
        false => format!("{}...", &name[0..42])
    }
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