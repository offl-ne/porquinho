mod cli;
mod dirs;
mod error;
mod file;
mod parser;
mod reader;
mod writer;

use std::path::{Path, PathBuf};

use bigdecimal::BigDecimal;
use chrono::{Datelike, Local};
use clap::Parser;
use dirs::Dirs;
pub use error::{Error, Result};
use parser::{Entry, EntryType};
use reader::Reader;

use crate::{
    cli::{Opts, Subcommand},
    file::BookkeepingFile,
    writer::Writer,
};

#[derive(Debug)]
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

struct GlobalState {
    opts: Opts,
    dirs: Dirs,
}

impl GlobalState {
    pub fn new() -> Result<Self> {
        let opts = Opts::parse();
        let dirs = Dirs::init()?;

        Ok(Self { opts, dirs })
    }

    pub fn run_command(self) -> Result<()> {
        let bk_path = self.bookkeeping_file_path();
        let day = Local::today().day() as u8;

        let (typ, decimal, description) = match self.opts.cmd {
            Subcommand::Take {
                amount,
                description,
            } => (EntryType::Debit, amount, description),
            Subcommand::Put {
                amount,
                description,
            } => (EntryType::Credit, amount, description),
            Subcommand::Status => {
                let total = Reader::new().total_from_file(&bk_path)?;
                println!("Status for {:?}", bk_path.file_name().unwrap());
                println!("\tIncoming: R$ {}", total.incoming);
                println!("\tOutgoing: R$ {}", total.outgoing);

                return Ok(());
            }
        };

        let entry = Entry {
            day,
            typ,
            decimal,
            description: &description,
        };

        Writer::write_entry(&bk_path, entry)?;

        Ok(())
    }

    fn bookkeeping_file_path(&self) -> PathBuf {
        self.dirs
            .data()
            .join(BookkeepingFile::current_file().as_path())
    }
}

fn exec() -> Result<()> {
    GlobalState::new()?.run_command()
}
