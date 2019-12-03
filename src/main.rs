/// floki - the development container launcher
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;

mod cli;
mod command;
mod config;
mod dind;
mod environment;
mod errors;
mod image;
mod interpret;
mod verify;
mod volumes;

use cli::{Cli, Subcommand};
use config::FlokiConfig;
use verify::verify_command;

use failure::Error;
use quicli::prelude::*;
use structopt::StructOpt;

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("floki")?;

    match run_floki_from_args(&args) {
        Ok(()) => (),
        Err(e) => {
            error!("A problem occured: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

/// Decide which commands to run given the input from the shell
fn run_floki_from_args(args: &Cli) -> Result<(), Error> {
    debug!("Got command line arguments: {:?}", &args);

    let environ = environment::Environment::gather(&args.config_file)?;
    debug!("Got environment {:?}", &environ);

    debug!("Selected configuration file: {:?}", &environ.config_file);

    let config = FlokiConfig::from_file(&environ.config_file)?;
    verify_command(args.local, &config)?;

    // Dispatch appropriate subcommand
    match &args.subcommand {
        // Pull the image in the configuration file
        Some(Subcommand::Pull {}) => {
            debug!("Trying to pull image {:?}", &config.image);
            debug!("Pulling image: {}", config.image.name()?);
            image::pull_image(&config.image.name()?)
        }

        // Run a command in the floki container
        Some(Subcommand::Run { command }) => {
            let inner_command = interpret::command_in_shell(config.shell.inner_shell(), &command);
            run_floki_container(&environ, &config, inner_command)
        }

        // Launch an interactive floki shell (the default)
        None => {
            let inner_command = config.shell.inner_shell().to_string();
            run_floki_container(&environ, &config, inner_command)
        }
    }
}

/// Launch a floki container running the inner command
fn run_floki_container(
    environ: &environment::Environment,
    config: &FlokiConfig,
    inner_command: String,
) -> Result<(), Error> {
    config.image.obtain_image()?;
    interpret::run_container(&environ, &config, &inner_command)
}
