use crate::command;
use crate::command::DockerCommandBuilder;
use crate::config::FlokiConfig;
use crate::dind::Dind;
use crate::environment::Environment;
use crate::errors;
use crate::volumes::resolve_volume_mounts;

use failure::Error;
use std::path;

/// Build a spec for the docker container, and then run it
pub(crate) fn run_container(
    environ: &Environment,
    config: &FlokiConfig,
    inner_command: &str,
) -> Result<(), Error> {
    let mount = get_mount_specification(&environ.floki_root, &config);
    let dind = Dind::new(config.dind.image(), mount);
    let mut cmd = command::DockerCommandBuilder::new(&config.image.name()?).add_volume(mount);

    let volumes = resolve_volume_mounts(
        &environ.config_file,
        &environ.floki_workspace,
        &config.volumes,
    );

    cmd = configure_dind(cmd, &config, &dind)?;
    cmd = configure_floki_user_env(cmd, &environ);
    cmd = configure_floki_host_mountdir_env(cmd, &environ.floki_root);
    cmd = configure_forward_user(cmd, &config, &environ);
    cmd = configure_forward_ssh_agent(cmd, &config, &environ)?;
    cmd = configure_docker_switches(cmd, &config)?;
    cmd = configure_working_directory(cmd, &environ, &config);
    cmd = configure_volumes(cmd, &volumes);

    instantiate_volumes(&volumes)?;
    let _handle = launch_dind_if_needed(&config, dind)?;
    let subshell_command = subshell_command(&config.init, inner_command);
    cmd.run(&[config.shell.outer_shell(), "-c", &subshell_command])
}

pub(crate) fn command_in_shell(shell: &str, command: &[String]) -> String {
    // Make sure our command runs in a subshell (we might switch user)
    let inner_shell: String = shell.to_string();
    inner_shell + " -c \"" + &command.join(" ") + "\""
}

fn configure_dind(
    cmd: DockerCommandBuilder,
    config: &FlokiConfig,
    dind: &Dind,
) -> Result<DockerCommandBuilder, Error> {
    if config.dind.enabled() {
        Ok(command::enable_docker_in_docker(cmd, dind)?)
    } else {
        Ok(cmd)
    }
}

fn configure_floki_user_env(cmd: DockerCommandBuilder, env: &Environment) -> DockerCommandBuilder {
    let user = env.user_details;
    let new_cmd = cmd.add_environment("FLOKI_HOST_UID", user.uid.to_string());
    new_cmd.add_environment("FLOKI_HOST_GID", user.gid.to_string())
}

fn configure_floki_host_mountdir_env(
    cmd: DockerCommandBuilder,
    floki_root: &path::Path,
) -> DockerCommandBuilder {
    cmd.add_environment("FLOKI_HOST_MOUNTDIR", floki_root)
}

fn configure_forward_user(
    cmd: DockerCommandBuilder,
    config: &FlokiConfig,
    env: &Environment,
) -> DockerCommandBuilder {
    if config.forward_user {
        let user = env.user_details;
        cmd.add_docker_switch("--user")
            .add_docker_switch(&format!("{}:{}", user.uid, user.gid))
    } else {
        cmd
    }
}

fn configure_forward_ssh_agent(
    cmd: DockerCommandBuilder,
    config: &FlokiConfig,
    env: &Environment,
) -> Result<DockerCommandBuilder, Error> {
    if config.forward_ssh_agent {
        if let Some(ref path) = env.ssh_agent_socket {
            Ok(command::enable_forward_ssh_agent(cmd, path))
        } else {
            Err(errors::FlokiError::NoSshAuthSock {}.into())
        }
    } else {
        Ok(cmd)
    }
}

fn configure_docker_switches(
    cmd: DockerCommandBuilder,
    config: &FlokiConfig,
) -> Result<DockerCommandBuilder, Error> {
    let mut cmd = cmd;
    for switch in decompose_switches(&config.docker_switches)? {
        cmd = cmd.add_docker_switch(switch);
    }
    Ok(cmd)
}

fn decompose_switches(
    specs: &Vec<String>,
) -> Result<Vec<String>, Error> {
    let mut flattened = Vec::new();

    for spec in specs {
        if let Some(switches) = shlex::split(spec) {
            for s in switches {
                flattened.push(s);
            }
        } else {
            Err(errors::FlokiError::MalformedDockerSwitch { item: spec.into() })?
        }
    }

    Ok(flattened)
}

fn configure_working_directory(
    cmd: DockerCommandBuilder,
    env: &Environment,
    config: &FlokiConfig,
) -> DockerCommandBuilder {
    cmd.set_working_directory(get_working_directory(
        &env.current_directory,
        &env.floki_root,
        &path::PathBuf::from(&config.mount),
    ))
}

/// Add mounts for each of the passed in volumes
fn configure_volumes(
    cmd: DockerCommandBuilder,
    volumes: &[(path::PathBuf, &path::PathBuf)],
) -> DockerCommandBuilder {
    let mut cmd = cmd; // Shadow as mutable
    for (src, dst) in volumes.iter() {
        cmd = cmd.add_volume((src, dst));
    }
    cmd
}

/// Create the backing directories for floki volumes if needed
fn instantiate_volumes(volumes: &[(path::PathBuf, &path::PathBuf)]) -> Result<(), Error> {
    for (src, _) in volumes.iter() {
        std::fs::create_dir_all(src)?;
    }
    Ok(())
}

/// Determine what directory we are currently in
fn get_working_directory(
    current_directory: &path::Path,
    floki_root: &path::Path,
    mount: &path::Path,
) -> path::PathBuf {
    mount.join(current_directory.strip_prefix(&floki_root).expect(
        "failed to deduce working directory - \
         floki_root should always be an ancestor of current_directory",
    ))
}

/// Specify the primary mount for the floki container
fn get_mount_specification<'a, 'b>(
    floki_root: &'a path::PathBuf,
    config: &'b FlokiConfig,
) -> (&'a path::PathBuf, &'b path::PathBuf) {
    (floki_root, &config.mount)
}

/// Turn the init section of a floki.yaml file into a command
/// that can be given to a shell
fn subshell_command(init: &[String], command: &str) -> String {
    let mut args: Vec<&str> = init.iter().map(|s| s as &str).collect::<Vec<&str>>();
    args.push(command);
    args.join(" && ")
}

/// Launch dind if specified by the configuration
fn launch_dind_if_needed(
    config: &FlokiConfig,
    dind: Dind,
) -> Result<Option<command::DaemonHandle>, Error> {
    if config.dind.enabled() {
        crate::dind::dind_preflight(config.dind.image())?;
        Ok(Some(dind.launch()?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_command_in_shell() {
        let subcommand = vec![String::from("foo"), String::from("bar")];

        let result = command_in_shell("bash", &subcommand);
        let expected = String::from("bash -c \"foo bar\"");

        assert!(result == expected);
    }

    #[test]
    fn test_get_working_directory() {
        let current_directory = path::PathBuf::from("/host/workingdir/");
        let floki_root = path::PathBuf::from("/host");
        let mount = path::PathBuf::from("/guest");

        assert!(
            get_working_directory(&current_directory, &floki_root, &mount)
                == path::PathBuf::from("/guest/workingdir/")
        )
    }

    #[test]
    fn test_decompose_switches() -> Result<(), Error> {
        let switches = vec!["-e FOO='bar baz'".to_string()];

        let want: Vec<String> = vec![
            "-e".to_string(),
	    "FOO=bar baz".to_string(),
        ];

	let got = decompose_switches(&switches)?;

        assert_eq!(want, got);

        Ok(())
    }

    #[test]
    fn test_decompose_switches_error() {
        let switches = vec!["-e FOO='bar baz".to_string()];
	let got = decompose_switches(&switches);
	assert!(got.is_err());
    }
}
