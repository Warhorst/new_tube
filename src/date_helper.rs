use std::cmp::Ordering;

use chrono::{DateTime, Local};

use crate::Video;

/// Check if the given date is after last_date, making it new.
/// The Strings are converted to dates, using the rfc3339 converter. Errors are
/// ignored, I just assume Youtube does it's job right.
pub fn date_is_new(last_date: &str, date: &str) -> bool {
    let last_date_time = DateTime::parse_from_rfc3339(last_date).unwrap();
    let date_time = DateTime::parse_from_rfc3339(date).unwrap();
    last_date_time < date_time
}

pub fn string_to_local_time_date(date: &str) -> DateTime<Local> {
    let utc_date_time = DateTime::parse_from_rfc3339(date).unwrap();
    DateTime::<Local>::from(utc_date_time)
}

pub fn compare_video_releases(v0: &Video, v1: &Video) -> Ordering {
    let dt0 = DateTime::parse_from_rfc3339(&v0.release_date).unwrap();
    let dt1 = DateTime::parse_from_rfc3339(&v1.release_date).unwrap();
    dt0.cmp(&dt1)
}