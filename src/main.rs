mod dirs;
mod error;
mod parser;
mod reader;

use std::path::Path;

use dirs::Dirs;
pub use error::{Error, Result};
use reader::Reader;

fn main() {
    if let Err(err) = exec() {
        eprintln!("Error: {}", err);
        std::process::exit(127);
    }
}

fn exec() -> Result<()> {
    let dirs = Dirs::init()?;
    let mut reader = Reader::new();
    reader.total_from_file("/home/vrmiguel/porquinho-test")?;

    Ok(())
}
