use std::{
    io::{Seek, Write},
    path::Path,
};

use fs_err as fs;
use toml::value::Value as TomlValue;

use crate::{
    parser::{Entry, EntryType},
    Bookkeeper, Result,
};

pub struct Writer;

impl Writer {
    pub fn write_entry(path: &Path, entry: Entry) -> Result<()> {
        let Bookkeeper { mut file, mut table, .. } = Bookkeeper::load_from_path(path)?;

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

// Truncates file to have only the written content
fn truncate_and_close_file(mut file: fs::File) -> Result<()> {
    let written_len = file.stream_position()?;
    file.set_len(written_len).map_err(Into::into)
}
