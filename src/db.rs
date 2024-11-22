use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};
use rusqlite::{params, Connection, Result, Row};

use crate::app::TimeKeeperError;

#[derive(Debug)]
pub struct Record {
    pub id: i32,
    pub check_in: NaiveTime,
    pub check_out: NaiveTime,
    pub date: NaiveDate,
}

impl Record {
    // Helper method to create Record from a database row
    fn from_row(row: &Row) -> Result<Record> {
        Ok(Record {
            id: row.get(0)?,
            check_in: NaiveTime::parse_from_str(&row.get::<_, String>(1)?, "%H:%M:%S").unwrap(),
            check_out: NaiveTime::parse_from_str(&row.get::<_, String>(2)?, "%H:%M:%S").unwrap(),
            date: NaiveDate::parse_from_str(&row.get::<_, String>(3)?, "%Y-%m-%d").unwrap(),
        })
    }
}

pub fn get_db_path() -> Result<PathBuf, TimeKeeperError> {
    let project_dirs = directories::ProjectDirs::from("", "", "timekeeper").ok_or_else(|| {
        TimeKeeperError::DatabaseError(rusqlite::Error::InvalidPath(PathBuf::from(
            "Could not determine project directory",
        )))
    })?;

    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir).map_err(|e| {
        TimeKeeperError::DatabaseError(rusqlite::Error::InvalidPath(PathBuf::from(format!(
            "Failed to create data directory: {}",
            e
        ))))
    })?;

    Ok(data_dir.join("keeper.db"))
}

fn get_connection() -> Result<Connection> {
    let db_path =
        get_db_path().map_err(|e| rusqlite::Error::InvalidPath(PathBuf::from(e.to_string())))?;
    Connection::open(db_path)
}

pub fn create_table() -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "
        Create table if not exists record (
            id integer primary key,
            check_in text,
            check_out text,
            date text
            )",
        (),
    )?;
    Ok(())
}

pub fn save_entry(record: &Record) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO record (check_in, check_out, date) VALUES (?1, ?2, ?3)",
        params![
            record.check_in.format("%H:%M:%S").to_string(),
            record.check_out.format("%H:%M:%S").to_string(),
            record.date.format("%Y-%m-%d").to_string(),
        ],
    )?;

    Ok(())
}

pub fn get_all_entries() -> Result<Vec<Record>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("SELECT * FROM record")?;

    let records = stmt
        .query_map([], |row| Record::from_row(row))?
        .collect::<Result<Vec<_>>>()?;

    Ok(records)
}

pub fn get_entries_by_date(date: NaiveDate) -> Result<Vec<Record>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("SELECT * FROM record WHERE date = ?")?;

    let date_str = date.format("%Y-%m-%d").to_string();
    let records = stmt
        .query_map([date_str], |row| Record::from_row(row))?
        .collect::<Result<Vec<_>>>()?;

    Ok(records)
}

pub fn update_entry(record: &Record) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "UPDATE record SET check_in = ?1, check_out = ?2, date = ?3 WHERE id = ?4",
        params![
            record.check_in.format("%H:%M:%S").to_string(),
            record.check_out.format("%H:%M:%S").to_string(),
            record.date.format("%Y-%m-%d").to_string(),
            record.id,
        ],
    )?;

    Ok(())
}

pub fn delete_entry(id: i32) -> Result<()> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM record WHERE id = ?1", params![id])?;
    Ok(())
}
