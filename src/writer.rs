use std::{
    io::{Read, Seek, Write},
    path::Path,
};

use crate::{
    parser::{Entry, EntryType},
    utils, Result,
};

use fs_err as fs;
use toml::value::Value as TomlValue;

pub struct Writer;

impl Writer {
    pub fn write_entry(path: &Path, entry: Entry) -> Result<()> {
        let (mut file, file_contents) = open_file(path)?;

        let mut table = utils::load_toml_table_or_default(&file_contents);

        let (array_key, kind_symbol) = match entry.kind {
            EntryType::Withdraw => ("take", '-'),
            EntryType::Deposit => ("put", '+'),
        };

        let line = format!(
            "{d} {k} {a} {D}",
            d = entry.day,
            k = kind_symbol,
            a = entry.amount,
            D = entry.description
        );

        table[array_key].as_array_mut().unwrap().push(line.into());

        let toml = toml::ser::to_string_pretty::<TomlValue>(&table.into()).unwrap();
        write!(file, "{}", toml)?;
        truncate_and_close_file(file)?;
        println!("Updated {}", path.display());

        Ok(())
    }
}

// Returns opened file and it's contents
//
// Seeked to the start, writes will overwrite from the start
fn open_file(path: &Path) -> Result<(fs::File, String)> {
    let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    file.rewind()?;

    Ok((file, content))
}

// Truncates file to have only the written content
fn truncate_and_close_file(mut file: fs::File) -> Result<()> {
    let written_len = file.stream_position()?;
    file.set_len(written_len).map_err(Into::into)
}
