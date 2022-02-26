mod dirs;
mod error;
mod file;
mod parser;
mod reader;

use bigdecimal::BigDecimal;
use dirs::Dirs;
pub use error::{Error, Result};
use reader::Reader;

use crate::file::BookkeepingFile;

pub struct Total {
    /// Amount spended
    pub outgoing: BigDecimal,
    /// Amount received
    pub incoming: BigDecimal,
}

fn main() {
    if let Err(err) = exec() {
        eprintln!("Error: {}", err);
        std::process::exit(127);
    }
}

fn exec() -> Result<()> {
    let _dirs = Dirs::init()?;
    let mut reader = Reader::new();
    reader.total_from_file("/home/vrmiguel/porquinho-test")?;

    dbg!(BookkeepingFile::current_file().as_path());

    Ok(())
}
