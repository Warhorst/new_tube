use rusqlite::Row;
use crate::Video;

pub struct Playlist {
    pub id: String,
    pub channel_name: String,
    pub last_video_name: String,
    pub last_video_id: String,
    pub last_video_release: String,
    pub last_video_duration: String
}

impl TryFrom<&Row<'_>> for Playlist {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Playlist {
            id: row.get(0)?,
            channel_name: row.get(1)?,
            last_video_name: row.get(2)?,
            last_video_id: row.get(3)?,
            last_video_release: row.get(4)?,
            last_video_duration: row.get(5)?
        })
    }
}

impl From<(&str, &Video)> for Playlist {
    fn from(id_video: (&str, &Video)) -> Self {
        let playlist_id = id_video.0;
        let video = id_video.1;

        Playlist {
            id: playlist_id.to_string(),
            channel_name: video.channel_name.clone(),
            last_video_name: video.name.clone(),
            last_video_id: video.id.clone(),
            last_video_release: video.release_date.clone(),
            last_video_duration: video.duration.clone()
        }
    }
}

impl Into<Video> for Playlist {
    fn into(self) -> Video {
        Video::new(
            self.id,
            self.channel_name,
            self.last_video_name,
            self.last_video_id,
            self.last_video_release,
            self.last_video_duration
        )
    }
}