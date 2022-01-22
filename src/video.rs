use cli_table::row::{Row, ToRow};
use serde::Serialize;

use crate::date::Date;

#[derive(Debug, Serialize)]
pub struct Video {
    pub playlist_id: String,
    pub channel_name: String,
    pub name: String,
    pub id: String,
    pub release_date: Date,
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
            release_date: Date::from_video_date(&release_date),
        }
    }

    pub fn is_new(&self, last_video_release: &str) -> bool {
        Date::from_db_playlist_date(last_video_release) < self.release_date
    }

    fn create_video_link(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }
}

impl ToRow<4> for Video {
    fn to_table_row(&self) -> Row<4> {
        Row::from([
            self.channel_name.clone(),
            shorten_video_name(self.name.clone()),
            self.create_video_link(),
            self.release_date.to_db_playlist_date()
        ])
    }
}

/// Names can get to long to display in a table cell. Limit to max 45 letters.
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