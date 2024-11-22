use crate::db::*;
use crate::parser::{get_today, parse_date_str, parse_time_str};
use chrono::{Duration, Local, NaiveDate};
use comfy_table::{Cell, Color, ContentArrangement, Row, Table};

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

pub fn display_summary() -> Result<(), TimeKeeperError> {
    let mut records = get_all_entries()?;

    if records.is_empty() {
        println!("No records found");
        return Ok(());
    }

    // Sort records in descending order by date and check-in time
    records.sort_by(|a, b| b.date.cmp(&a.date).then(b.check_in.cmp(&a.check_in)));

    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(Row::from(vec![
            Cell::new("Date").fg(Color::Cyan),
            Cell::new("Check-in").fg(Color::Green),
            Cell::new("Check-out").fg(Color::Red),
            Cell::new("Duration").fg(Color::Yellow),
        ]));

    let mut total_duration = Duration::zero();
    let mut current_date: Option<NaiveDate> = None;
    let mut date_duration = Duration::zero();

    for record in &records {
        let duration = record.check_out.signed_duration_since(record.check_in);
        total_duration = total_duration + duration;

        // If we're on a new date, add a subtotal for the previous date
        if let Some(prev_date) = current_date {
            if prev_date != record.date && date_duration.num_minutes() > 0 {
                table.add_row(vec![
                    Cell::new("Subtotal").fg(Color::Blue),
                    Cell::new("").fg(Color::Blue),
                    Cell::new("").fg(Color::Blue),
                    Cell::new(format!(
                        "{}h {}m",
                        date_duration.num_minutes() / 60,
                        date_duration.num_minutes() % 60
                    ))
                    .fg(Color::Blue),
                ]);
                table.add_row(vec!["", "", "", ""]); // Empty row as separator
                date_duration = Duration::zero();
            }
        }

        date_duration = date_duration + duration;
        current_date = Some(record.date);

        let hours = duration.num_minutes() / 60;
        let minutes = duration.num_minutes() % 60;
        let duration_str = format!("{}h {}m", hours, minutes);

        table.add_row(vec![
            record.date.format("%Y-%m-%d").to_string(),
            record.check_in.format("%H:%M").to_string(),
            record.check_out.format("%H:%M").to_string(),
            duration_str,
        ]);
    }

    // Add final date subtotal if there are records
    if let Some(_) = current_date {
        if date_duration.num_minutes() > 0 {
            table.add_row(vec![
                Cell::new("Subtotal").fg(Color::Blue),
                Cell::new("").fg(Color::Blue),
                Cell::new("").fg(Color::Blue),
                Cell::new(format!(
                    "{}h {}m",
                    date_duration.num_minutes() / 60,
                    date_duration.num_minutes() % 60
                ))
                .fg(Color::Blue),
            ]);
        }
    }

    // Add grand total if there are multiple records
    if records.len() > 1 {
        table.add_row(vec!["", "", "", ""]); // Empty row as separator
        table.add_row(vec![
            Cell::new("Total").fg(Color::Magenta),
            Cell::new("").fg(Color::Magenta),
            Cell::new("").fg(Color::Magenta),
            Cell::new(format!(
                "{}h {}m",
                total_duration.num_minutes() / 60,
                total_duration.num_minutes() % 60
            ))
            .fg(Color::Magenta),
        ]);
    }

    println!("All Records:");
    println!("{table}");

    Ok(())
}
