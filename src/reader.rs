use std::{path::Path, str};

use bigdecimal::{BigDecimal, Zero};
use fixed_buffer::{deframe_line, FixedBuf};
use fs_err as fs;

use crate::{
    parser::{Entry, EntryType},
    Result, Total,
};

/// A stack-based file reader
pub struct Reader {
    buf: FixedBuf<512>,
}

impl Reader {
    pub const fn new() -> Self {
        Self {
            buf: FixedBuf::new(),
        }
    }

    /// Read a bookkeeping file and return the total amount spent and received.
    pub fn total_from_file(&mut self, path: impl AsRef<Path>) -> Result<Total> {
        let mut file = fs::File::open(path.as_ref())?;
        let mut outgoing = BigDecimal::zero();
        let mut incoming = BigDecimal::zero();

        while let Ok(Some(line)) = self.buf.read_frame(&mut file, deframe_line) {
            let line = str::from_utf8(line)?;
            let entry = Entry::from_str(line)?;
            match entry.kind {
                EntryType::Debit => outgoing += entry.amount,
                EntryType::Credit => incoming += entry.amount,
            }
        }

        Ok(Total { outgoing, incoming })
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, str::FromStr};

    use bigdecimal::BigDecimal;
    use tempfile::NamedTempFile;

    use crate::reader::Reader;

    #[test]
    fn reads_total_from_file_correctly() {
        let mut dummy = NamedTempFile::new().unwrap();
        writeln!(dummy, "22 + 200.50 Payment").unwrap();
        writeln!(dummy, "22 + 300.25 Another Payment").unwrap();
        writeln!(dummy, "23 - 10.25 Lunch").unwrap();
        writeln!(dummy, "23 - 10.27 Dinner").unwrap();

        let mut reader = Reader::new();
        let total = reader.total_from_file(dummy.path()).unwrap();

        assert_eq!(total.incoming, BigDecimal::from_str("500.75").unwrap());
        assert_eq!(total.outgoing, BigDecimal::from_str("20.52").unwrap());
    }
}
