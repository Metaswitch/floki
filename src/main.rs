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
mod volumes;

use cli::{Cli, Subcommand};
use config::FlokiConfig;

use failure::Error;
use quicli::prelude::*;
use structopt::StructOpt;

fn main() -> CliResult {
    let args = Cli::from_args();
    configure_logging(args.verbosity)?;

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
    if args.local {
        warn!("-l/--local is deprecated and may be removed in a future release");
    }

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

        Some(Subcommand::Completion { shell }) => {
            Ok(Cli::clap().gen_completions_to("floki", *shell, &mut std::io::stdout()))
        }

        Some(Subcommand::Volume {}) => {
            list_volumes(&environ, &config);
            Ok(())
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
    config.image.obtain_image(&environ.floki_root)?;
    interpret::run_container(&environ, &config, &inner_command)
}

/// Print the volumes used in the current configuration
fn list_volumes(environ: &environment::Environment, config: &FlokiConfig) {
    println!("{:20} {:20} {}", "NAME", "MOUNT", "HOSTPATH");
    for (name, volume) in config.volumes.iter() {
        let hostpath =
            volumes::cache_path(&environ.floki_workspace, &environ.config_file, name, volume);
        let mount = &volume.mount;
        println!(
            "{:20} {:20} {}",
            name,
            mount.to_string_lossy(),
            hostpath.to_string_lossy()
        );
    }
}

/// Configure the logger
fn configure_logging(verbosity: u8) -> Result<(), Error> {
    let level = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => Err(errors::FlokiUserError::InvalidVerbositySetting { setting: verbosity })?,
    };
    simplelog::TermLogger::init(
        level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
    )?;
    Ok(())
}
