use bigdecimal::{BigDecimal, Zero};

use toml::value::Table as TomlTable;

use crate::{
    error::Result,
    parser::{Entry, EntryType},
};

// TODO: add more processed information to this
#[derive(Debug)]
pub(super) struct BookkeeperStatus {
    /// Total amount spended
    pub outgoing_total: BigDecimal,
    /// Total amount received
    pub incoming_total: BigDecimal,
}

impl BookkeeperStatus {
    pub(super) fn display(&self) {
        let balance = &self.incoming_total - &self.outgoing_total;

        println!("\tIncoming: R$ {}", self.incoming_total);
        println!("\tOutgoing: R$ {}", self.outgoing_total);
        println!("\tBalance:  R$ {}", balance);
    }

    pub(super) fn from_toml_table(table: &TomlTable) -> Result<Self> {
        let mut outgoing_total = BigDecimal::zero();
        let mut incoming_total = BigDecimal::zero();

        let (take, put) = (
            table["take"].as_array().unwrap(),
            table["put"].as_array().unwrap(),
        );

        for entry in take.iter().chain(put) {
            let entry = entry.as_str().unwrap();
            let entry = Entry::from_str(entry).unwrap();

            match entry.kind {
                EntryType::Withdraw => outgoing_total += entry.amount,
                EntryType::Deposit => incoming_total += entry.amount,
            }
        }

        Ok(Self { outgoing_total, incoming_total })
    }
}
