mod status;

use std::{
    io::{Read, Seek, Write},
    path::PathBuf,
};

use fs_err as fs;
use toml::value::{Table as TomlTable, Value as TomlValue};

use crate::{
    error::{Error, Result, TomlTypeCheck, TomlTypeCheckDiagnosis},
    parser::Operation,
};

use status::BookkeeperStatus;

pub struct Bookkeeper {
    pub file: fs::File,
    pub file_path: PathBuf,
    pub file_contents: String,
    pub table: TomlTable,
    status: BookkeeperStatus,
}

impl Bookkeeper {
    pub fn display_status(&self) {
        // Safety: Always has file name because it's in format "MM-YYYY"
        println!("Status for {:?}", self.file_path.file_name().unwrap());

        self.status.display();
    }

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

    pub fn add_operation(&mut self, operation: Operation) -> Result<()> {
        let (array_key, kind_symbol) = operation.kind.name_and_symbol();

        let line = format!(
            "{d} {k} {a} {D}",
            d = operation.day,
            k = kind_symbol,
            a = operation.amount,
            D = operation.description
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

    fn load_toml_table_or_default(input_text: &str) -> TomlTable {
        let toml = if input_text.trim().is_empty() {
            generate_default_toml()
        } else {
            input_text.parse().unwrap()
        };

        unwrap_toml_table(toml)
    }

    fn status_from_toml_table(table: &TomlTable) -> Result<BookkeeperStatus> {
        BookkeeperStatus::from_toml_table(table)
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
    toml::toml! {
        take = []
        put = []
    }
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

        assert_eq!(status.put_total, BigDecimal::from_str("500.75").unwrap());
        assert_eq!(status.take_total, BigDecimal::from_str("420.52").unwrap());
    }
}

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
