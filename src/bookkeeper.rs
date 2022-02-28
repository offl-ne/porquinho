use std::{
    io::{Read, Seek},
    path::Path,
};

use crate::{
    bookkeeper,
    error::{Error, Result, TomlTypeCheck, TomlTypeCheckDiagnosis},
};

use fs_err as fs;
use toml::value::{Table as TomlTable, Value as TomlValue};

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

/// Loads toml table from text and type check, or generate default
///
/// # Errors:
///
/// Returns `Error::InvalidTomlTypes` if loaded toml has incorrect types.
pub fn load_toml_table_or_default(input_text: &str) -> (TomlTable, TomlTypeCheckDiagnosis) {
    let toml = if input_text.trim().is_empty() {
        generate_default_toml()
    } else {
        input_text.parse().unwrap()
    };

    let unwrap_toml_table = |toml: TomlValue| -> TomlTable {
        match toml {
            TomlValue::Table(table) => table,
            _ => unreachable!(),
        }
    };

    let table = unwrap_toml_table(toml);
    let type_check_diagnosis = type_check_toml_fields(&table);
    (table, type_check_diagnosis)
}

// Returns opened file and it's contents
//
// Seeked to the start, writes will overwrite from the start
pub fn open_file_and_moar(path: &Path) -> Result<(fs::File, String, TomlTable)> {
    let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    file.rewind()?;

    let (table, type_check_diagnosis) = bookkeeper::load_toml_table_or_default(&file_contents);

    if type_check_diagnosis.has_error_description() {
        Err(Error::InvalidTomlTypes {
            description: type_check_diagnosis.into_inner(),
            path: path.into(),
        })
    } else {
        Ok((file, file_contents, table))
    }
}
