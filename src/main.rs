mod bookkeeper;
mod cli;
mod dirs;
mod error;
mod file;
mod parser;

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
};

fn main() {
    if let Err(err) = exec() {
        eprintln!("Error: {}", err);
        std::process::exit(127);
    }
}

fn exec() -> Result<()> {
    GlobalState::new()?.run_command()
}

struct GlobalState {
    cmd: Subcommand,
    bookkeeper: Bookkeeper,
}

impl GlobalState {
    pub fn new() -> Result<Self> {
        let cmd = Opts::parse().cmd;
        let dirs = Dirs::init()?;

        let bk_path = dirs.data().join(BookkeepingFile::current_file().as_path());
        create_file_if_not_existent(&bk_path);
        let bookkeeper = Bookkeeper::load_from_path(bk_path)?;

        Ok(Self { cmd, bookkeeper })
    }

    pub fn run_command(self) -> Result<()> {
        let day = Local::today().day() as u8;
        let Self { cmd, mut bookkeeper } = self;

        match cmd {
            Subcommand::Take { amount, ref description } => {
                let entry = Entry::new(day, EntryType::Withdraw, amount, description);
                bookkeeper.add_entry(entry)?;
            }
            Subcommand::Put { amount, ref description } => {
                let entry = Entry::new(day, EntryType::Deposit, amount, description);
                bookkeeper.add_entry(entry)?;
            }
            Subcommand::Status => {
                bookkeeper.display_status();
            }
        };

        Ok(())
    }
}
