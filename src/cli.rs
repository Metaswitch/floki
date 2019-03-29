/// Description of the CLI interface to floki
use quicli::prelude::*;

/// Subcommands of the main floki command
#[derive(Debug, StructOpt)]
pub(crate) enum Subcommand {

    /// Run a command within the container
    #[structopt(name = "run")]
    Run { command: Vec<String> },

    /// Pull the image in the configuration file
    #[structopt(name = "pull")]
    Pull{}
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
    #[structopt(flatten)]
    pub(crate) verbosity: Verbosity,
    #[structopt(subcommand)]
    pub(crate) subcommand: Option<Subcommand>,
}
