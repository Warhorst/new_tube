use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct Video {
    pub channel_name: String,
    pub name: String,
    pub id: String,
    pub release_date: DateTime<Local>,
}

impl Video {
    /// Create a Video from the given parameters. The release_date must be a valid
    /// RFC 3339 date string (see https://www.ietf.org/rfc/rfc3339.txt).
    pub fn new(
        channel_name: String,
        name: String,
        id: String,
        release_date: String,
    ) -> Self {
        Video {
            channel_name, name, id,
            release_date: DateTime::parse_from_rfc3339(&release_date).unwrap().with_timezone(&Local)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::video::Video;

    #[test]
    fn create_video_works() {
        Video::new(
            "Channel".to_string(),
            "Video".to_string(),
            "69NICE420".to_string(),
            "2021-12-19T19:13:00Z".to_string()
        );
    }
}