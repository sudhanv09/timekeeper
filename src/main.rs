use clap::Parser;
use std::fs;
use timekeeper::{app::*, db::*, parser::*};

#[derive(Parser, Debug)]
struct Args {
    check_in: Option<String>,

    check_out: Option<String>,

    #[arg(short, long)]
    date: Option<String>,
}

fn ensure_db_exists() -> Result<(), TimeKeeperError> {
    if !fs::metadata("keeper.db").is_ok() {
        create_table().map_err(TimeKeeperError::from)?;
    }
    Ok(())
}

fn main() -> Result<(), TimeKeeperError> {
    ensure_db_exists()?;

    let args = Args::parse();
    match (args.check_in, args.check_out) {
        (Some(time), None) => {
            handle_check_in(&time, args.date)?;
        }
        (None, Some(time)) => {
            handle_check_out(&time, args.date)?;
        }
        (Some(check_in), Some(check_out)) => {
            handle_record(&check_in, &check_out, args.date)?;
        }
        (None, None) => {
            // Show today's records if no arguments provided
            let records = get_entries_by_date(get_today())?;
            if records.is_empty() {
                println!("No records for today");
            } else {
                println!("Today's records:");
                for record in records {
                    println!(
                        "Check-in: {}, Check-out: {}, Duration: {} minutes",
                        record.check_in.format("%H:%M"),
                        record.check_out.format("%H:%M"),
                        record
                            .check_out
                            .signed_duration_since(record.check_in)
                            .num_minutes()
                    );
                }
            }
        }
    }

    Ok(())
}
