use std::{
    io::{Read, Seek, Write},
    path::PathBuf,
};

use bigdecimal::{BigDecimal, Zero};
use fs_err as fs;
use toml::value::{Table as TomlTable, Value as TomlValue};

use crate::{
    error::{Error, Result, TomlTypeCheck, TomlTypeCheckDiagnosis},
    parser::{Entry, EntryType},
};

#[derive(Debug)]
pub struct BookkeeperStatus {
    /// Total amount spended
    pub outgoing_total: BigDecimal,
    /// Total amount received
    pub incoming_total: BigDecimal,
    // /// Each spend transaction
    // pub outgoing: Vec<BigDecimal>,
    // /// Each receive transaction
    // pub incoming: Vec<BigDecimal>,
}

impl BookkeeperStatus {
    pub fn display(&self) {
        println!("\tIncoming: R$ {}", self.incoming_total);
        println!("\tOutgoing: R$ {}", self.outgoing_total);
    }
}

pub struct Bookkeeper {
    pub file: fs::File,
    pub file_path: PathBuf,
    pub file_contents: String,
    pub table: TomlTable,
    pub status: BookkeeperStatus,
}

impl Bookkeeper {
    pub fn display_status(&self) {
        // Safety: Always has file name because it's in format "MM-YYYY"
        println!("Status for {:?}", self.file_path.file_name().unwrap());

        self.status.display();
    }

    // Returns opened file and it's contents
    //
    // Seeked to the start, writes will overwrite from the start
    pub fn load_from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let mut file = fs::OpenOptions::new().read(true).write(true).open(&path)?;
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)?;
        file.rewind()?;

        let table = Self::load_toml_table_or_default(&file_contents);

        let type_check_diagnosis = type_check_toml_fields(&table);
        if type_check_diagnosis.has_error_description() {
            return Err(Error::InvalidTomlTypes {
                description: type_check_diagnosis.into_inner(),
                path,
            });
        }

        let status = Self::status_from_toml_table(&table)?;

        Ok(Self {
            file,
            file_path: path,
            file_contents,
            table,
            status,
        })
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<()> {
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

        self.table[array_key]
            .as_array_mut()
            .unwrap()
            .push(line.into());

        let temporary_toml = TomlValue::Table(std::mem::take(&mut self.table));
        let toml = toml::ser::to_string_pretty::<TomlValue>(&temporary_toml).unwrap();
        self.table = unwrap_toml_table(temporary_toml);
        write!(self.file, "{}", toml)?;
        truncate_and_close_file(&mut self.file)?;
        println!("Updated {}", self.file_path.display());

        Ok(())
    }

    /// Loads toml table from text and type check, or generate default
    ///
    /// # Errors:
    ///
    /// Returns `Error::InvalidTomlTypes` if loaded toml has incorrect types.
    fn load_toml_table_or_default(input_text: &str) -> TomlTable {
        let toml = if input_text.trim().is_empty() {
            generate_default_toml()
        } else {
            input_text.parse().unwrap()
        };

        unwrap_toml_table(toml)
    }

    /// Read a bookkeeping file and return the total amount spent and received.
    pub fn status_from_toml_table(table: &TomlTable) -> Result<BookkeeperStatus> {
        let mut outgoing_total = BigDecimal::zero();
        let mut incoming_total = BigDecimal::zero();

        let (take, put) = (
            table["take"].as_array().unwrap(),
            table["put"].as_array().unwrap(),
        );

        for entry in take.iter().chain(put) {
            let entry = entry.as_str().unwrap();
            let entry = Entry::from_str(entry).unwrap();

            match entry.kind {
                EntryType::Withdraw => outgoing_total += entry.amount,
                EntryType::Deposit => incoming_total += entry.amount,
            }
        }

        Ok(BookkeeperStatus { outgoing_total, incoming_total })
    }
}

fn type_check_toml_fields(table: &TomlTable) -> TomlTypeCheckDiagnosis {
    let is_take_array = table.get("take").map_or(false, TomlValue::is_array);
    let is_put_array = table.get("put").map_or(false, TomlValue::is_array);
    let is_target_int_or_undefined = table.get("target").map_or(true, TomlValue::is_integer);

    let is_array_of_strings = |array_value: Option<&TomlValue>| {
        array_value
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .all(|value| value.is_str())
    };

    let is_take_array_of_strings = is_take_array && is_array_of_strings(table.get("take"));
    let is_put_array_of_strings = is_put_array && is_array_of_strings(table.get("put"));

    let toml_type_check = TomlTypeCheck {
        is_take_array,
        is_put_array,
        is_target_int_or_undefined,
        is_take_array_of_strings,
        is_put_array_of_strings,
    };

    toml_type_check.into_diagnosis()
}

pub fn generate_default_toml() -> TomlValue {
    // // Qual dos códigos é melhor? com ou sem a macro?
    let toml = toml::toml! {
        take = []
        put = []
    };

    // // Qual dos códigos é melhor? com ou sem a macro?
    // let mut map = TomlTable::new();
    // map.insert("take".to_string(), TomlValue::Array(vec![]));
    // map.insert("put".to_string(), TomlValue::Array(vec![]));

    toml
}

#[cfg(test)]
mod tests {
    use std::{io::Write, str::FromStr};

    use bigdecimal::BigDecimal;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn reads_income_and_outcome_total_from_file_correctly() {
        let mut dummy = NamedTempFile::new().unwrap();

        let toml = toml::toml! {
            put = [
                "22 + 200.50 Payment",
                "22 + 300.25 Another Payment",
            ]
            take = [
                "23 - 10.25 Lunch",
                "23 - 10.27 Dinner",
                "24 - 400.00 kindle-para-bish",
            ]
        };
        writeln!(dummy, "{}", toml).unwrap();

        let bookkeeper = Bookkeeper::load_from_path(dummy.path()).unwrap();
        let status = bookkeeper.status;

        assert_eq!(
            status.incoming_total,
            BigDecimal::from_str("500.75").unwrap()
        );
        assert_eq!(
            status.outgoing_total,
            BigDecimal::from_str("420.52").unwrap()
        );
    }
}

// Truncates file to have only the written content
fn truncate_and_close_file(file: &mut fs::File) -> Result<()> {
    let written_len = file.stream_position()?;
    file.set_len(written_len).map_err(Into::into)
}

fn unwrap_toml_table(toml: TomlValue) -> TomlTable {
    match toml {
        TomlValue::Table(table) => table,
        _ => unreachable!(),
    }
}
