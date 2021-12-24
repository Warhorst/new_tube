use chrono::{Date, DateTime, Local, NaiveTime};
use cli_table::row::{Row, ToRow};

#[derive(Debug)]
pub struct Video {
    pub channel_name: String,
    pub name: String,
    pub id: String,
    pub release_date: Date<Local>,
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
            release_date: DateTime::parse_from_rfc3339(&release_date).unwrap().with_timezone(&Local).date()
        }
    }

    /// Return if a video was released today
    pub fn is_new(&self) -> bool {
        let today_at_midnight = Local::today().and_time(NaiveTime::from_hms(0, 0, 0)).unwrap().date();
        self.release_date > today_at_midnight
    }

    fn create_video_link(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }
}

impl ToRow<4> for Video {
    fn to_table_row(&self) -> Row<4> {
        Row::from([
            self.channel_name.clone(),
            self.name.clone(),
            self.create_video_link(),
            self.release_date.to_string()
        ])
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