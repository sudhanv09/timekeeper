use crate::db::*;
use crate::parser::{get_today, parse_date_str, parse_time_str};
use chrono::{Local, NaiveDate};

#[derive(Debug)]
pub enum TimeKeeperError {
    DatabaseError(rusqlite::Error),
    InvalidTime(String),
    CheckOutBeforeCheckIn,
    NoCheckInRecord,
    ParseError(String),
}

impl From<rusqlite::Error> for TimeKeeperError {
    fn from(err: rusqlite::Error) -> Self {
        TimeKeeperError::DatabaseError(err)
    }
}

pub fn handle_check_in(time_str: &str, date: Option<String>) -> Result<(), TimeKeeperError> {
    let check_in = parse_time_str(time_str)?;
    let date = match date {
        Some(date_str) => NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|_| TimeKeeperError::ParseError("Invalid date format".to_string()))?,
        None => get_today(),
    };

    // Create a new record with check_out as None
    let record = Record {
        id: 0,
        check_in,
        check_out: check_in, // Temporary value, will be updated on check-out
        date,
    };

    save_entry(&record).map_err(TimeKeeperError::from)?;
    println!("Checked in at {}", check_in.format("%H:%M"));
    Ok(())
}

pub fn handle_check_out(time_str: &str, date: Option<String>) -> Result<(), TimeKeeperError> {
    let check_out = parse_time_str(time_str)?;
    let date = match date {
        Some(date_str) => NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|_| TimeKeeperError::ParseError("Invalid date format".to_string()))?,
        None => get_today(),
    };

    // Get the latest record for today
    let records = get_entries_by_date(date).map_err(TimeKeeperError::from)?;
    let latest_record = records.last().ok_or(TimeKeeperError::NoCheckInRecord)?;

    // Verify check-out time is after check-in
    if check_out <= latest_record.check_in {
        return Err(TimeKeeperError::CheckOutBeforeCheckIn);
    }

    // Update the record with check-out time
    let updated_record = Record {
        id: latest_record.id,
        check_in: latest_record.check_in,
        check_out,
        date,
    };

    update_entry(&updated_record).map_err(TimeKeeperError::from)?;

    // Calculate duration
    let duration = check_out
        .signed_duration_since(latest_record.check_in)
        .num_minutes();

    println!("Checked out at {}", check_out.format("%H:%M"));
    println!(
        "Total time: {} hours {} minutes",
        duration / 60,
        duration % 60
    );
    Ok(())
}

pub fn handle_record(
    check_in_str: &str,
    check_out_str: &str,
    date_str: Option<String>,
) -> Result<(), TimeKeeperError> {
    let check_in = parse_time_str(check_in_str)?;
    let check_out = parse_time_str(check_out_str)?;

    if check_out <= check_in {
        return Err(TimeKeeperError::CheckOutBeforeCheckIn);
    }

    let date = match date_str {
        Some(date_str) => parse_date_str(&date_str)?,
        None => Local::now().date_naive(),
    };

    let record = Record {
        id: 0,
        check_in,
        check_out,
        date,
    };

    save_entry(&record)?;

    let duration = check_out.signed_duration_since(check_in).num_minutes();
    println!("Saved record for {}:", date.format("%Y-%m-%d"));
    println!("  Check-in:  {}", check_in.format("%H:%M"));
    println!("  Check-out: {}", check_out.format("%H:%M"));
    println!(
        "  Duration:  {} hours {} minutes",
        duration / 60,
        duration % 60
    );

    Ok(())
}
