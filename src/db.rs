use chrono::{NaiveDate, NaiveTime};
use rusqlite::{params, Connection, Result, Row};

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

pub fn create_table() -> Result<()> {
    let conn = Connection::open("keeper.db")?;

    conn.execute(
        "
        Create table record (
            id integer primary key,
            check_in text,
            check_out text,
            date text
            )",
        (),
    )?;
    Ok(())
}

// Create (Save) a new entry
pub fn save_entry(record: &Record) -> Result<()> {
    let conn = Connection::open("keeper.db")?;

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

// Read all entries
pub fn get_all_entries() -> Result<Vec<Record>> {
    let conn = Connection::open("keeper.db")?;
    let mut stmt = conn.prepare("SELECT * FROM record")?;

    let records = stmt
        .query_map([], |row| Record::from_row(row))?
        .collect::<Result<Vec<_>>>()?;

    Ok(records)
}

// Read entries for a specific date
pub fn get_entries_by_date(date: NaiveDate) -> Result<Vec<Record>> {
    let conn = Connection::open("keeper.db")?;
    let mut stmt = conn.prepare("SELECT * FROM record WHERE date = ?")?;

    let date_str = date.format("%Y-%m-%d").to_string();
    let records = stmt
        .query_map([date_str], |row| Record::from_row(row))?
        .collect::<Result<Vec<_>>>()?;

    Ok(records)
}

// Update an existing entry
pub fn update_entry(record: &Record) -> Result<()> {
    let conn = Connection::open("keeper.db")?;

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

// Delete an entry
pub fn delete_entry(id: i32) -> Result<()> {
    let conn = Connection::open("keeper.db")?;
    conn.execute("DELETE FROM record WHERE id = ?1", params![id])?;
    Ok(())
}
