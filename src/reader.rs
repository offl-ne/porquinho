/////
use std::io::Read;
use std::path::Path;

use bigdecimal::{BigDecimal, Zero};
use fs_err as fs;

use crate::{
    parser::{Entry, EntryType},
    utils, Result, Status,
};

/// Read a bookkeeping file and return the total amount spent and received.
pub fn status_from_file(path: &Path) -> Result<Status> {
    let mut file = fs::File::open(path)?;

    let mut outgoing = BigDecimal::zero();
    let mut incoming = BigDecimal::zero();

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let table = utils::load_toml_table_or_default(&content);

    utils::type_check_toml_fields(&table);

    let (take, put) = (
        table["take"].as_array().unwrap(),
        table["put"].as_array().unwrap(),
    );

    for entry in take.iter().chain(put) {
        let entry = entry.as_str().unwrap();
        let entry = Entry::from_str(entry).unwrap();

        match entry.kind {
            EntryType::Withdraw => outgoing += entry.amount,
            EntryType::Deposit => incoming += entry.amount,
        }
    }

    Ok(Status { outgoing, incoming })
}

#[cfg(test)]
mod tests {
    use std::{io::Write, str::FromStr};

    use bigdecimal::BigDecimal;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn reads_total_from_file_correctly() {
        let mut dummy = NamedTempFile::new().unwrap();

        let toml = toml::toml! {
            put = [
                "22 + 200.50 Payment",
                "22 + 300.25 Another Payment",
            ]
            take = [
                "23 - 10.25 Lunch",
                "23 - 10.27 Dinner",
                "24 - 400.00 kindle-para-bish",
            ]
        };
        writeln!(dummy, "{}", toml).unwrap();

        let status = status_from_file(dummy.path()).unwrap();

        assert_eq!(status.incoming, BigDecimal::from_str("500.75").unwrap());
        assert_eq!(status.outgoing, BigDecimal::from_str("420.52").unwrap());
    }
}
