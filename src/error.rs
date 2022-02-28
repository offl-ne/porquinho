use std::{io, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't find a home directorys")]
    NoValidHomeDirFound,
    #[error("Failed to create folder: {0}")]
    CouldNotCreateFolder(PathBuf),
    #[error("Filesystem error: {0}")]
    FileSystem(#[from] io::Error),
    #[error("Parsing error: {0}")]
    Parse(#[from] crate::parser::ParseError),
    #[error("Invalid UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Toml values are in ??? unexpected types: {description}. at {path}.")]
    InvalidTomlTypes { description: String, path: PathBuf },
}

pub struct TomlTypeCheck {
    pub is_take_array: bool,
    pub is_put_array: bool,
    pub is_target_int_or_undefined: bool,
    pub is_take_array_of_strings: bool,
    pub is_put_array_of_strings: bool,
}

pub struct TomlTypeCheckDiagnosis(String);

impl TomlTypeCheckDiagnosis {
    pub fn has_error_description(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl TomlTypeCheck {
    pub fn into_diagnosis(self) -> TomlTypeCheckDiagnosis {
        // comma separated description of check failure reasons
        let mut description = String::new();

        // if boolean is true, append text to the description, separating with commas
        let mut describe = |boolean, text| {
            if boolean {
                if !description.is_empty() {
                    description.push_str(", ");
                }
                description.push_str(text);
            }
        };

        describe(!self.is_take_array, "");
        describe(!self.is_put_array, "");
        describe(!self.is_target_int_or_undefined, "target is not a integer");

        describe(
            self.is_take_array && !self.is_take_array_of_strings,
            "take array contains a non-string element",
        );
        describe(
            self.is_put_array && !self.is_put_array_of_strings,
            "put array contains a non-string element",
        );

        TomlTypeCheckDiagnosis(description)
    }
}
