use toml::value::{Table as TomlTable, Value as TomlValue};

pub fn unwrap_toml_table(toml: TomlValue) -> TomlTable {
    match toml {
        TomlValue::Table(table) => table,
        _ => unreachable!(),
    }
}

pub fn type_check_toml_fields(table: &TomlTable) {
    let is_take_array = table.get("take").map(TomlValue::is_array).unwrap_or(false);
    let is_put_array = table.get("put").map(TomlValue::is_array).unwrap_or(false);
    let is_target_int_or_undefined = table.get("target").map_or(true, TomlValue::is_integer);

    let is_ok = is_take_array && is_put_array && is_target_int_or_undefined;

    if !is_ok {
        panic!("type check failed: {:?}", table);
    }
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

pub fn load_toml_table_or_default(input_text: &str) -> TomlTable {
    let toml = if input_text.trim().is_empty() {
        generate_default_toml()
    } else {
        input_text.parse().unwrap()
    };

    let table = unwrap_toml_table(toml);
    type_check_toml_fields(&table);
    table
}
