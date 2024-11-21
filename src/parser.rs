use crate::app::TimeKeeperError;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};

impl std::fmt::Display for TimeKeeperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeKeeperError::DatabaseError(e) => write!(f, "Database error: {}", e),
            TimeKeeperError::InvalidTime(time) => write!(f, "Invalid time format: {}", time),
            TimeKeeperError::CheckOutBeforeCheckIn => {
                write!(f, "Check-out time before check-in time")
            }
            TimeKeeperError::NoCheckInRecord => write!(f, "No check-in record found"),
            TimeKeeperError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for TimeKeeperError {}

pub fn parse_date_str(date_str: &str) -> Result<NaiveDate, TimeKeeperError> {
    let cleaned_date = date_str.replace('/', "");

    if cleaned_date.len() != 4 || !cleaned_date.chars().all(|c| c.is_digit(10)) {
        return Err(TimeKeeperError::ParseError(
            "Invalid date format. Use MMDD or MM/DD".to_string(),
        ));
    }

    let month: u32 = cleaned_date[0..2]
        .parse()
        .map_err(|_| TimeKeeperError::ParseError("Invalid month".to_string()))?;
    let day: u32 = cleaned_date[2..4]
        .parse()
        .map_err(|_| TimeKeeperError::ParseError("Invalid day".to_string()))?;

    let current_year = Local::now().year();

    NaiveDate::from_ymd_opt(current_year, month, day).ok_or_else(|| {
        TimeKeeperError::ParseError(format!("Invalid date: month={}, day={}", month, day))
    })
}

pub fn parse_time_str(val: &str) -> Result<NaiveTime, TimeKeeperError> {
    let time_str = val.to_lowercase();

    // Handle military time
    if let Ok(time) = NaiveTime::parse_from_str(&time_str, "%H:%M") {
        return Ok(time);
    }

    // compact military time 1900
    if time_str.len() == 4 && time_str.chars().all(|c| c.is_digit(10)) {
        if let Ok(time) =
            NaiveTime::parse_from_str(&format!("{}:{}", &time_str[0..2], &time_str[2..4]), "%H:%M")
        {
            return Ok(time);
        }
    }

    // Handle am/pm
    if time_str.ends_with("am") || time_str.ends_with("pm") {
        let meridian = &time_str[time_str.len() - 2..];
        let time_part = &time_str[..time_str.len() - 2];

        // Handle cases without colon (e.g., "9am", "730pm", "0730pm")
        let normalized_time = if !time_part.contains(':') {
            if time_part.len() <= 2 {
                // Simple hour format (e.g., "9am")
                format!("{}:00 {}", time_part, meridian)
            } else {
                // Handle compact time (e.g., "730pm")
                let hour_part = if time_part.len() == 3 {
                    &time_part[..1]
                } else {
                    &time_part[..2]
                };
                let minute_part = &time_part[time_part.len() - 2..];
                format!("{}:{} {}", hour_part, minute_part, meridian)
            }
        } else {
            // Already has colon (e.g., "7:30pm")
            format!("{} {}", time_part, meridian)
        };

        if let Ok(mut time) = NaiveTime::parse_from_str(&normalized_time, "%I:%M %P") {
            if time.hour() == 12 && meridian == "am" {
                time = NaiveTime::from_hms_opt(0, time.minute(), 0).unwrap();
            }
            return Ok(time);
        }
    }
    Err(TimeKeeperError::ParseError(format!(
        "Invalid time format '{}'. Use HH:MM (24-hour) or HHMM(am/pm) like 530pm",
        val
    )))
}

pub fn get_today() -> NaiveDate {
    Local::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    #[test]
    fn test_military_time() {
        // Test standard military time formats
        assert_eq!(
            parse_time_str("19:00").unwrap(),
            NaiveTime::from_hms_opt(19, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("07:30").unwrap(),
            NaiveTime::from_hms_opt(7, 30, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("00:00").unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("23:59").unwrap(),
            NaiveTime::from_hms_opt(23, 59, 0).unwrap()
        );
    }

    #[test]
    fn test_am_times() {
        // Test various AM time formats
        assert_eq!(
            parse_time_str("9am").unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("9:00am").unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("09:00am").unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("7:30am").unwrap(),
            NaiveTime::from_hms_opt(7, 30, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("07:30am").unwrap(),
            NaiveTime::from_hms_opt(7, 30, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("12:00am").unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_pm_times() {
        // Test various PM time formats
        assert_eq!(
            parse_time_str("9pm").unwrap(),
            NaiveTime::from_hms_opt(21, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("9:00pm").unwrap(),
            NaiveTime::from_hms_opt(21, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("09:00pm").unwrap(),
            NaiveTime::from_hms_opt(21, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("7:30pm").unwrap(),
            NaiveTime::from_hms_opt(19, 30, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("07:30pm").unwrap(),
            NaiveTime::from_hms_opt(19, 30, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("12:00pm").unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_compact_formats() {
        // Test compact time formats without colons
        assert_eq!(
            parse_time_str("0900").unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("1900").unwrap(),
            NaiveTime::from_hms_opt(19, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("730pm").unwrap(),
            NaiveTime::from_hms_opt(19, 30, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("0730pm").unwrap(),
            NaiveTime::from_hms_opt(19, 30, 0).unwrap()
        );
    }

    #[test]
    fn test_case_insensitivity() {
        // Test case insensitivity
        assert_eq!(
            parse_time_str("9AM").unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("9PM").unwrap(),
            NaiveTime::from_hms_opt(21, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("9:00AM").unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            parse_time_str("9:00PM").unwrap(),
            NaiveTime::from_hms_opt(21, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_invalid_formats() {
        // Test invalid time formats
        assert!(parse_time_str("25:00").is_err()); // Invalid hour
        assert!(parse_time_str("13pm").is_err()); // Invalid hour for PM
        assert!(parse_time_str("9:60").is_err()); // Invalid minutes
        assert!(parse_time_str("abc").is_err()); // Invalid format
        assert!(parse_time_str("").is_err()); // Empty string
        assert!(parse_time_str("9:00xyz").is_err()); // Invalid suffix
    }
}
