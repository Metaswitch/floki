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
use dind::Dind;
use quicli::prelude::*;
use std::process::ExitStatus;


/// Turn the init section of a floki.yaml file into a command
/// that can be given to a shell
fn subshell_command(init: &Vec<String>, command: &str) -> String {
    let mut args = init.clone();
    args.push(command.into());
    args.join(" && ")
}

/// Obtain information for a volume bind of the current working directory
fn mount_current_spec(host_directory: &str, mount_directory: &str) -> (String, String) {
    (host_directory.to_string(), mount_directory.to_string())
}

/// Build a spec for the docker container, and then run it
fn run_container(config: &FlokiConfig, command: &str) -> Result<ExitStatus> {
    // Gather information from the users environment
    let environ = environment::Environment::gather()?;

    // Get the mount locations.
    let mount = mount_current_spec(
        &environ.current_directory,
        &config.mount
    );

    // Assign a container for docker-in-docker - we don't spawn it yet
    let mut dind = Dind::new(&mount);

    let image = config.image.obtain_image()?;

    let mut cmd = command::DockerCommandBuilder::new(&image, config.shell.outer_shell()).add_volume(&mount);

    cmd = interpret::configure_dind(cmd, &config, &mut dind)?;
    cmd = interpret::configure_floki_user_env(cmd, &environ);
    cmd = interpret::configure_forward_user(cmd, &config, &environ);
    cmd = interpret::configure_forward_ssh_agent(cmd, &config, &environ)?;
    cmd = interpret::configure_docker_switches(cmd, &config);
    cmd = interpret::configure_working_directory(cmd, &config);

    Ok(cmd.run(subshell_command(&config.init, command))?)
}

/// Decide which commands to run given the input from the shell
fn run_floki_from_args(args: &Cli) -> Result<()> {
    debug!("Got command line arguments: {:?}", &args);

    let config = FlokiConfig::from_file(&args.config_file)?;
    debug!("Got configuration {:?}", &config);

    verify_command(&args, &config)?;

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
            run_container(&config, config.shell.inner_shell())
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
