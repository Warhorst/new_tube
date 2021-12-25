use std::cmp::Ordering;
use chrono::{Datelike, DateTime, Timelike, TimeZone, Utc};

/// A simplified date to be used when comparing video releases
#[derive(Debug, Eq, PartialEq)]
pub struct Date {
    year: usize,
    month: usize,
    day: usize,
    hour: usize,
    minute: usize,
    second: usize,
}

impl Date {
    pub fn from_video_date(date_string: &String) -> Self {
        let date_time = DateTime::parse_from_rfc3339(date_string).unwrap().with_timezone(&Utc);
        Date {
            year: date_time.year() as usize,
            month: date_time.month() as usize,
            day: date_time.day() as usize,
            hour: date_time.hour() as usize,
            minute: date_time.minute() as usize,
            second: date_time.second() as usize,
        }
    }

    pub fn from_db_playlist_date(date_string: &String) -> Self {
        let parts = date_string.split(":").map(|s| s.parse::<usize>().unwrap()).collect::<Vec<_>>();
        Date {
            year: parts[0],
            month: parts[1],
            day: parts[2],
            hour: parts[3],
            minute: parts[4],
            second: parts[5],
        }
    }

    pub fn to_db_playlist_date(&self) -> String {
        format!("{}:{}:{}:{}:{}:{}",
                self.year,
                Self::get_formatted(self.month),
                Self::get_formatted(self.day),
                Self::get_formatted(self.hour),
                Self::get_formatted(self.minute),
                Self::get_formatted(self.second))
    }

    /// Ensure a value has a leading zero, if it is less than 10
    fn get_formatted(value: usize) -> String {
        format!("{:0>2}", value)
    }

    fn to_date_time(&self) -> DateTime<Utc> {
        Utc.ymd(
            self.year as i32,
            self.month as u32,
            self.day as u32,
        ).and_hms(
            self.hour as u32,
            self.minute as u32,
            self.second as u32,
        )
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_date_time().partial_cmp(&other.to_date_time())
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_date_time().cmp(&other.to_date_time())
    }
}

#[cfg(test)]
mod tests {
    use crate::date::Date;

    #[test]
    pub fn from_video_date_works() {
        let date = Date::from_video_date(&"2021-12-19T19:13:00Z".to_string());
        assert_eq!(date, expected_date())
    }

    #[test]
    pub fn from_db_playlist_date_works() {
        let date = Date::from_db_playlist_date(&"2021:12:19:19:13:00".to_string());
        assert_eq!(date, expected_date())
    }

    #[test]
    pub fn to_db_playlist_date_works() {
        let date_string = "2021:12:19:19:13:00".to_string();
        let date = Date::from_db_playlist_date(&date_string);
        assert_eq!(date.to_db_playlist_date(), date_string)
    }

    #[test]
    pub fn cmp_works() {
        assert!(Date::from_db_playlist_date(&"2021:12:19:19:13:00".to_string()) < Date::from_db_playlist_date(&"2021:12:20:19:13:00".to_string()))
    }

    fn expected_date() -> Date {
        Date {
            year: 2021,
            month: 12,
            day: 19,
            hour: 19,
            minute: 13,
            second: 0,
        }
    }
}