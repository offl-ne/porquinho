use bigdecimal::BigDecimal;
use clap::Parser;

/// Simplistic personal finances helper
///
/// Repository: https://github.com/vrmiguel/porquinho
#[derive(Parser, Debug)]
#[clap(about, version)]
pub struct Opts {
    #[clap(subcommand)]
    pub cmd: Subcommand,
}

#[derive(Parser, PartialEq, Eq, Debug)]
pub enum Subcommand {
    /// Record a new withdraw from your account
    Take {
        #[clap(required = true)]
        amount: BigDecimal,

        #[clap(required = true)]
        description: String,
    },
    /// Record a new deposit to your account
    Put {
        #[clap(required = true)]
        amount: BigDecimal,

        #[clap(required = true)]
        description: String,
    },
    /// Current status for your account
    Status,
}
