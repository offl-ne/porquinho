use std::{io::Write, path::Path};

use crate::{
    parser::{Entry, EntryType},
    Result,
};

use fs_err as fs;

pub struct Writer;

impl Writer {
    pub fn write_entry(path: &Path, entry: Entry) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?;

        let typ = match entry.typ {
            EntryType::Debit => "-",
            EntryType::Credit => "+",
        };

        writeln!(
            file,
            "{d} {t} {a} {D}",
            d = entry.day,
            t = typ,
            a = entry.decimal,
            D = entry.description
        )?;

        println!("Updated {}", path.display());

        Ok(())
    }
}
