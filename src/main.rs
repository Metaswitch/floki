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

use cli::{Cli, Subcommand};
use config::FlokiConfig;
use dind::Dind;
use quicli::prelude::*;
use std::process::ExitStatus;


/// Turn the init section of a floki.yaml file into a command
/// that can be given to a shell
fn subshell_command(config: &FlokiConfig, command: &String) -> String {
    let mut args = config.init.clone();
    args.push(command.clone());
    args.join(" && ")
}

/// Obtain information for a volume bind of the current working directory
fn mount_current_spec(host_directory: &str, mount_directory: &str) -> (String, String) {
    (host_directory.to_string(), mount_directory.to_string())
}

/// Build a spec for the docker container, and then run it
fn run_container(config: &FlokiConfig, command: &String) -> Result<ExitStatus> {
    // Gather information from the users environment
    let environ = environment::Environment::gather()?;

    // Get the mount locations.
    let mount = mount_current_spec(
        &environ.current_directory,
        &config.mount_pwd
    );

    // Assign a container for docker-in-docker - we don't spawn it yet
    let mut dind = Dind::new(&mount);

    let image = config.image.obtain_image()?;

    let mut cmd = command::DockerCommandBuilder::new(&image, config.shell.outer_shell()).add_volume(&mount);

    if config.dind {
        cmd = command::enable_docker_in_docker(cmd, &mut dind)?;
    }

    let (user, group) = environ.user_details;

    cmd = cmd.add_environment(&("FLOKI_HOST_UID".to_string(), user.clone()));
    cmd = cmd.add_environment(&("FLOKI_HOST_GID".to_string(), group.clone()));

    if config.forward_user {
        cmd = cmd.add_docker_switch(&format!("--user {}:{}", user, group));
    }

    if config.forward_ssh_agent {
        if let Some(path) = environ.ssh_agent_socket {
            cmd = command::enable_forward_ssh_agent(cmd, &path)?;
        } else {
            Err(errors::FlokiError::NoSshAuthSock {})?
        }
    }

    for switch in &config.docker_switches {
        cmd = cmd.add_docker_switch(&switch);
    }

    Ok(cmd.run(subshell_command(&config, command))?)
}

/// Decide which commands to run given the input from the shell
fn run_floki_from_args(args: &Cli) -> Result<()> {
    debug!("Got command line arguments: {:?}", &args);

    let config = FlokiConfig::from_file(&args.config_file)?;
    debug!("Got configuration {:?}", &config);

    // Dispatch depending on whether we have received a subcommand
    let exit_status = match &args.subcommand {
        // If we pull an image, we don't run a container - do an early return
        Some(Subcommand::Pull {}) => {
            debug!("Trying to pull image {:?}", &config.image);
            debug!("Pulling image: {}", config.image.name());
            return image::pull_image(config.image.name());
        }

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
