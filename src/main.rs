/// floki - the development container launcher
#[macro_use]
extern crate quicli;
#[macro_use]
extern crate failure;
extern crate serde_yaml;
extern crate uuid;

mod cli;
mod command;
mod config;
mod dind;
mod environment;
mod errors;
mod image;
mod verify;
mod interpret;

use cli::{Cli, Subcommand};
use config::FlokiConfig;
use verify::verify_command;
use quicli::prelude::*;


main!(
    |args: Cli, log_level: verbosity| match run_floki_from_args(&args) {
        Ok(()) => (),
        Err(e) => {
            error!("A problem occured: {}", e);
            std::process::exit(1);
        }
    }
);


/// Decide which commands to run given the input from the shell
fn run_floki_from_args(args: &Cli) -> Result<()> {
    debug!("Got command line arguments: {:?}", &args);

    let config = FlokiConfig::from_file(&args.config_file)?;
    debug!("Got configuration {:?}", &config);

    verify_command(&args, &config)?;

    // Dispatch appropriate subcommand
    match &args.subcommand {
        // Pull the image in the configuration file
        Some(Subcommand::Pull {}) => {
            debug!("Trying to pull image {:?}", &config.image);
            debug!("Pulling image: {}", config.image.name());
            image::pull_image(config.image.name())
        }

        // Run a command in the floki container
        Some(Subcommand::Run { command }) => {
            let inner_command = interpret::command_in_shell(config.shell.inner_shell(), &command);
            run_floki_container(&config, inner_command)
        }

        // Launch an interactive floki shell (the default)
        None => {
            let inner_command = config.shell.inner_shell().to_string();
            run_floki_container(&config, inner_command)
        }
    }
}


/// Launch a floki container running the inner command
fn run_floki_container(config: &FlokiConfig, inner_command: String) -> Result<()> {
    let environ = environment::Environment::gather()?;
    debug!("Got environment {:?}", &environ);

    config.image.obtain_image()?;

    let subshell_command = command::subshell_command(&config.init, inner_command);
    debug!("Running container with command '{}'", &subshell_command);
    interpret::run_container(&config, &environ, &subshell_command)
}
