/// floki - the development container launcher
#[macro_use]
extern crate quicli;
#[macro_use]
extern crate serde_derive;
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

use cli::{Cli, Subcommand};
use config::FlokiConfig;
use dind::Dind;
use quicli::prelude::*;
use std::env::current_dir;
use std::fs::File;
use std::io::Read;
use std::process::ExitStatus;

/// Obtain configuration from a file (which is possibly specified
/// on the command line)
fn load_config_from_file(args: &Cli) -> Result<FlokiConfig> {
    let mut f = File::open(args.config_file.clone()).map_err(|e| {
        errors::FlokiError::ProblemOpeningConfigYaml {
            name: args.config_file.clone(),
            error: e,
        }
    })?;
    let mut raw = String::new();
    f.read_to_string(&mut raw)
        .map_err(|e| errors::FlokiError::ProblemReadingConfigYaml {
            name: args.config_file.clone(),
            error: e,
        })?;
    let config =
        serde_yaml::from_str(&raw).map_err(|e| errors::FlokiError::ProblemParsingConfigYaml {
            name: args.config_file.clone(),
            error: e,
        })?;
    Ok(config)
}

/// Turn the init section of a floki.yaml file into a command
/// that can be given to a shell
fn subshell_command(config: &FlokiConfig, command: &String) -> String {
    let mut args = config.init.clone();
    args.push(command.clone());
    args.join(" && ")
}

/// Get the current working directory as a String
fn get_current_working_directory() -> Result<String> {
    Ok(format!("{}", current_dir()?.display()))
}

/// Obtain information for a volume bind of the current working directory
fn mount_current_spec(config: &FlokiConfig) -> Result<(String, String)> {
    Ok((get_current_working_directory()?, config.mount_pwd.clone()))
}

/// Build a spec for the docker container, and then run it
fn run_container(config: &FlokiConfig, command: &String) -> Result<ExitStatus> {
    // Get the mount locations.
    let mount = mount_current_spec(&config)?;

    // Assign a container for docker-in-docker - we don't spawn it yet
    let mut dind = Dind::new(&mount);

    let image = config.image.obtain_image()?;

    let mut cmd = command::DockerCommandBuilder::new(&image, config.shell.outer_shell()).add_volume(&mount);

    if config.dind {
        cmd = command::enable_docker_in_docker(cmd, &mut dind)?;
    }

    let (user, group) = environment::get_user_details()?;

    cmd = cmd.add_environment(&("FLOKI_HOST_UID".to_string(), user.clone()));
    cmd = cmd.add_environment(&("FLOKI_HOST_GID".to_string(), group.clone()));

    if config.forward_user {
        cmd = cmd.add_docker_switch(&format!("--user {}:{}", user, group));
    }

    if config.forward_ssh_agent {
        cmd = command::enable_forward_ssh_agent(cmd)?;
    }

    if config.forward_tmux_socket {
        match command::enable_forward_tmux_socket(cmd.clone()) {
            Ok(c) => cmd = c,
            Err(e) => warn!("Could not forward tmux socket - continuing anyway: {}", e),
        }
    }

    for switch in &config.docker_switches {
        cmd = cmd.add_docker_switch(&switch);
    }

    Ok(cmd.run(subshell_command(&config, command))?)
}

/// Try and pull an image, given configuration (fails if we have a
/// build specification)
fn run_image_pull(config: &FlokiConfig) -> Result<()> {
    match config.image {
        image::Image::Build { build: _ } => Err(errors::FlokiError::ImageNotPullable {})?,
        _ => {
            if config.image.will_pull() {
                warn!(
                    "{} will be pulled anyway - you've specified this in your configuration",
                    config.image.name()
                );
                Ok(())
            } else {
                debug!("Pulling image: {}", config.image.name());
                image::pull_image(config.image.name())?;
                Ok(())
            }
        }
    }
}

/// Decide which commands to run given the input from the shell
fn run_floki_from_args(args: &Cli) -> Result<()> {
    debug!("Got command line arguments: {:?}", &args);

    let config = load_config_from_file(&args)?;
    debug!("Got configuration {:?}", &config);

    if args.pull {
        debug!("Trying to pull image {:?}", &config.image);
        run_image_pull(&config)?;
    }

    // Dispatch depending on whether we have received a subcommand
    let exit_status = match &args.subcommand {
        Some(Subcommand::Run { command }) => {
            // Make sure our command runs in a subshell (we might switch user)
            let inner_shell: String = config.shell.inner_shell().into();
            let command_string = inner_shell + " -c \"" + &command.join(" ") + "\"";
            debug!("Running container with command '{}'", &command_string);
            run_container(&config, &command_string)
        }
        _ => {
            debug!("Running container");
            run_container(&config, &config.shell.inner_shell().into())
        }
    }?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(errors::FlokiError::RunContainerFailed {
            exit_status: errors::FlokiSubprocessExitStatus {
                process_description: "docker run".into(),
                exit_status: exit_status,
            },
        })?
    }
}

main!(
    |args: Cli, log_level: verbosity| match run_floki_from_args(&args) {
        Ok(()) => (),
        Err(e) => {
            error!("A problem occured: {}", e);
            std::process::exit(1);
        }
    }
);
