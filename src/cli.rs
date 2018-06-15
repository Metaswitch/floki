/// Description of the CLI interface to floki
use quicli::prelude::*;

/// Subcommands of the main floki command
#[derive(Debug, StructOpt)]
pub(crate) enum Subcommand {
    #[structopt(name = "run")]
    Run { command: Vec<String> },
}

/// Main CLI interface
#[derive(Debug, StructOpt)]
#[structopt(
    name = "floki",
    about = "The interactive container launcher."
)]
pub(crate) struct Cli {
    #[structopt(long = "config", short = "c", default_value = "floki.yaml")]
    pub(crate) config_file: String,
    #[structopt(
        long = "pull",
        help = "Update the image in your configuration file"
    )]
    pub(crate) pull: bool,
    #[structopt(flatten)]
    pub(crate) verbosity: Verbosity,
    #[structopt(subcommand)]
    pub(crate) subcommand: Option<Subcommand>,
}
