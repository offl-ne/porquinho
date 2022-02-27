use bigdecimal::BigDecimal;
use clap::{Parser, ValueHint};

#[derive(Parser, Debug)]
#[clap(about, version)]
/// Simplistic personal finances helper
///
/// Repository: https://github.com/vrmiguel/porquinho
pub struct Opts {
    #[clap(subcommand)]
    pub cmd: Subcommand,
}

#[derive(Parser, PartialEq, Eq, Debug)]
pub enum Subcommand {
    /// Record a debit transaction from your account
    Take {
        #[clap(required = true)]
        amount: BigDecimal,

        #[clap(required = true)]
        description: String,
    },
    /// Record a new credit to your account
    Put {
        #[clap(required = true)]
        amount: BigDecimal,

        #[clap(required = true)]
        description: String,
    },
    /// Current status for your
    Status {},
}
