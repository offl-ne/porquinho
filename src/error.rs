use std::io;
use std::path::PathBuf;

use crate::parser::ParseError;

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
    Parse(#[from] ParseError),
    #[error("Invalid UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}
