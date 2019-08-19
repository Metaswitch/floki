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

use cli::{Cli, Subcommand};
use config::FlokiConfig;
use verify::verify_command;

use failure::Error;
use quicli::prelude::*;
use std::path;
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

    let environ = environment::Environment::gather()?;
    debug!("Got environment {:?}", &environ);

    let (floki_root, config_file) = match &args.config_file {
        Some(config_file) => (environ.current_directory.to_path_buf(), config_file.clone()),
        None => {
            let config_file = find_floki_yaml(&environ.current_directory)?;
            (
                config_file
                    .parent()
                    .expect(
                        "failed to select config file - config_file should always have a parent",
                    )
                    .to_path_buf(),
                config_file,
            )
        }
    };
    debug!("Selected configuration file: {:?}", &config_file);

    let config = FlokiConfig::from_file(&config_file)?;
    verify_command(args.local, &config)?;

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
            run_floki_container(&environ, &floki_root, &config, inner_command)
        }

        // Launch an interactive floki shell (the default)
        None => {
            let inner_command = config.shell.inner_shell().to_string();
            run_floki_container(&environ, &floki_root, &config, inner_command)
        }
    }
}

/// Launch a floki container running the inner command
fn run_floki_container(
    environ: &environment::Environment,
    floki_root: &path::Path,
    config: &FlokiConfig,
    inner_command: String,
) -> Result<(), Error> {
    config.image.obtain_image()?;

    let subshell_command = command::subshell_command(&config.init, inner_command);
    debug!("Running container with command '{}'", &subshell_command);
    interpret::run_container(&environ, &floki_root, &config, &subshell_command)
}

/// Search all ancestors of the current directory for a floki.yaml file
/// name.
fn find_floki_yaml(current_directory: &path::Path) -> Result<path::PathBuf, Error> {
    current_directory
        .ancestors()
        .map(|a| a.join("floki.yaml"))
        .find(|f| f.is_file())
        .ok_or(errors::FlokiError::ProblemFindingConfigYaml {}.into())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use tempdir;

    fn touch_file(path: &path::Path) -> Result<(), Error> {
        fs::create_dir_all(
            path.parent()
                .ok_or(format_err!("Unable to take parent of path"))?,
        )?;
        fs::OpenOptions::new().create(true).write(true).open(path)?;
        Ok(())
    }

    #[test]
    fn test_find_floki_yaml_current_dir() -> Result<(), Error> {
        let tmp_dir = tempdir::TempDir::new("")?;
        let floki_yaml_path = tmp_dir.path().join("floki.yaml");
        touch_file(&floki_yaml_path)?;
        assert_eq!(find_floki_yaml(&tmp_dir.path())?, floki_yaml_path);
        Ok(())
    }

    #[test]
    fn test_find_floki_yaml_ancestor() -> Result<(), Error> {
        let tmp_dir = tempdir::TempDir::new("")?;
        let floki_yaml_path = tmp_dir.path().join("floki.yaml");
        touch_file(&floki_yaml_path)?;
        assert_eq!(
            find_floki_yaml(&tmp_dir.path().join("dir/subdir"))?,
            floki_yaml_path
        );
        Ok(())
    }

    #[test]
    fn test_find_floki_yaml_sibling() -> Result<(), Error> {
        let tmp_dir = tempdir::TempDir::new("")?;
        let floki_yaml_path = tmp_dir.path().join("src/floki.yaml");
        touch_file(&floki_yaml_path)?;
        assert!(find_floki_yaml(&tmp_dir.path().join("include")).is_err());
        Ok(())
    }
}
