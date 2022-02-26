use std::{path::Path, str};

use bigdecimal::{BigDecimal, Zero};
use fixed_buffer::{deframe_line, FixedBuf};
use fs_err as fs;

use crate::{
    parser::{Entry, EntryType},
    Result,
};

// TODO: not the correct file for this
pub struct Total {
    pub outgoing: BigDecimal,
    pub incoming: BigDecimal,
}

pub struct Reader {
    buf: FixedBuf<512>,
}

impl Reader {
    pub const fn new() -> Self {
        Self {
            buf: FixedBuf::new(),
        }
    }

    pub fn total_from_file(&mut self, path: impl AsRef<Path>) -> Result<Total> {
        let mut file = fs::File::open(path.as_ref())?;
        let mut outgoing = BigDecimal::zero();
        let mut incoming = BigDecimal::zero();

        while let Ok(Some(line)) = self.buf.read_frame(&mut file, deframe_line) {
            let line = str::from_utf8(line)?;
            let entry = Entry::from_str(line)?;
            match entry.typ {
                EntryType::Debit => outgoing += entry.decimal,
                EntryType::Credit => incoming += entry.decimal,
            }
        }

        Ok(Total { outgoing, incoming })
    }
}
