mod bookkeeper;
mod cli;
mod dirs;
mod error;
mod file;
mod parser;
mod reader;
mod writer;

use std::path::PathBuf;

use bigdecimal::BigDecimal;
use chrono::{Datelike, Local};
use clap::Parser;
use dirs::Dirs;
use parser::{Entry, EntryType};

pub use crate::{
    bookkeeper::Bookkeeper,
    error::{Error, Result},
};

use crate::{
    cli::{Opts, Subcommand},
    file::create_file_if_not_existent,
    file::BookkeepingFile,
    writer::Writer,
};

#[derive(Debug)]
pub struct Status {
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
    // Bookkeeping path
    bk_path: PathBuf,
}

impl GlobalState {
    pub fn new() -> Result<Self> {
        let opts = Opts::parse();
        let dirs = Dirs::init()?;

        let bk_path = dirs.data().join(BookkeepingFile::current_file().as_path());
        create_file_if_not_existent(&bk_path);

        Ok(Self { opts, bk_path })
    }

    pub fn run_command(self) -> Result<()> {
        let day = Local::today().day() as u8;
        let Self {
            ref bk_path,
            opts: Opts { cmd },
            ..
        } = self;

        match cmd {
            Subcommand::Take { amount, ref description } => {
                let entry = Entry::new(day, EntryType::Withdraw, amount, description);
                Writer::write_entry(bk_path, entry)?;
            }
            Subcommand::Put { amount, ref description } => {
                let entry = Entry::new(day, EntryType::Deposit, amount, description);
                Writer::write_entry(bk_path, entry)?;
            }
            Subcommand::Status => {
                let total = reader::status_from_file(bk_path)?;
                // Safeyu: Always has file name because it's in format "MM-YYYY"
                println!("Status for {:?}", bk_path.file_name().unwrap());
                println!("\tIncoming: R$ {}", total.incoming);
                println!("\tOutgoing: R$ {}", total.outgoing);
            }
        };

        Ok(())
    }
}

fn exec() -> Result<()> {
    GlobalState::new()?.run_command()
}
