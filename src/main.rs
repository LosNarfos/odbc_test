//! A program executing a query and printing the result as csv to standard out. Requires
//! `anyhow` and `csv` crate.

#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

//use anyhow::Error;
use odbc_api::{Connection, Error, IntoParameter, buffers::BufferDesc};
use odbc_api::{buffers::TextRowSet, Cursor, Environment, ConnectionOptions, ResultSetMetadata};
use std::{ffi::CStr, io::{stdout, Write},path::PathBuf};

/// Maximum number of rows fetched with one row set. Fetching batches of rows is usually much
/// faster than fetching individual rows.
const BATCH_SIZE: usize = 5000;

fn insert_birth_years(conn: &Connection, names: &[&str], years: &[i16]) -> Result<(), Error> {

    // All columns must have equal length.
    assert_eq!(names.len(), years.len());

    let prepared = conn.prepare("INSERT INTO Birthdays (name, year) VALUES (?, ?)")?;

    // Create a columnar buffer which fits the input parameters.
    let buffer_description = [
        BufferDesc::Text { max_str_len: 255 },
        BufferDesc::I16 { nullable: false },
    ];
    // The capacity must be able to hold at least the largest batch. We do everything in one go, so
    // we set it to the length of the input parameters.
    let capacity = names.len();
    // Allocate memory for the array column parameters and bind it to the statement.
    let mut prebound = prepared.into_column_inserter(capacity, buffer_description)?;
    // Length of this batch
    prebound.set_num_rows(capacity);


    // Fill the buffer with values column by column
    let mut col = prebound
        .column_mut(0)
        .as_text_view()
        .expect("We know the name column to hold text.");

    for (index, name) in names.iter().enumerate() {
        col.set_cell(index, Some(name.as_bytes()));
    }

    let col = prebound
        .column_mut(1)
        .as_slice::<i16>()
        .expect("We know the year column to hold i16.");
    col.copy_from_slice(years);

    prebound.execute()?;
    Ok(())
}


fn main() -> Result<(), Error> {

    let env = Environment::new()?;
    let connection_string = "
        Driver={ODBC Driver 17 for SQL Server};\
        ConnSettings=SET CLIENT_ENCODING TO 'UTF8';\
        Server=SQLDBSRV11\\INST2;\
        Database=ECAD_PARTS_dev;\
        UID=ecad_user;\
        PWD=E34Corona;\
    ";

    let connection = env.connect_with_connection_string(connection_string, ConnectionOptions::default())?;

    connection.execute("TRUNCATE TABLE dbo.Birthdays", ())?;

    let names = ["ä", "Peter", "Max"];
    let years = [1988, 1987, 1986];
    insert_birth_years(&connection, &names, &years).unwrap();

    let value = 10;

    //connection.execute("INSERT INTO [dbo].[Birthdays] ([CDB No]) VALUES (?)", ())?;

    Ok(())
}
