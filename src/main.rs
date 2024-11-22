use clap::Parser;
use std::fs;
use timekeeper::{app::*, db::*};

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
        (None, None) => display_summary()?,
    }

    Ok(())
}
