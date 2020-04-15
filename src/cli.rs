/// Description of the CLI interface to floki
use std::path;
use structopt::StructOpt;

/// Subcommands of the main floki command
#[derive(Debug, StructOpt)]
pub(crate) enum Subcommand {
    /// Run a command within the container
    #[structopt(name = "run")]
    Run { command: Vec<String> },

    /// Pull the image in the configuration file
    #[structopt(name = "pull")]
    Pull {},
}

/// Main CLI interface
#[derive(Debug, StructOpt)]
#[structopt(name = "floki", about = "The interactive container launcher.")]
pub(crate) struct Cli {
    /// Use the specified config instead of searching the tree for a
    /// "floki.yaml" file.
    #[structopt(long = "config", short = "c")]
    pub(crate) config_file: Option<path::PathBuf>,

    /// Run floki regardless of reproducibility
    #[structopt(long = "local", short = "l")]
    pub(crate) local: bool,

    #[structopt(subcommand)]
    pub(crate) subcommand: Option<Subcommand>,
}
