use crate::command;
use crate::command::DockerCommandBuilder;
use crate::config::FlokiConfig;
use crate::dind::Dind;
use crate::environment::Environment;
use crate::errors;

use failure::Error;
use std::path;

/// Build a spec for the docker container, and then run it
pub(crate) fn run_container(
    environ: &Environment,
    floki_root: &path::Path,
    config: &FlokiConfig,
    command: &str,
) -> Result<(), Error> {
    let (mut cmd, mut dind) = build_basic_command(&floki_root, &config)?;

    cmd = configure_dind(cmd, &config, &mut dind)?;
    cmd = configure_floki_user_env(cmd, &environ);
    cmd = configure_floki_host_mountdir_env(cmd, &floki_root);
    cmd = configure_forward_user(cmd, &config, &environ);
    cmd = configure_forward_ssh_agent(cmd, &config, &environ)?;
    cmd = configure_docker_switches(cmd, &config);
    cmd = configure_working_directory(cmd, &environ, &floki_root, &config);

    cmd.run(command)
}

pub(crate) fn command_in_shell(shell: &str, command: &Vec<String>) -> String {
    // Make sure our command runs in a subshell (we might switch user)
    let inner_shell: String = shell.to_string();
    let command_string = inner_shell + " -c \"" + &command.join(" ") + "\"";
    command_string
}

fn configure_dind(
    cmd: DockerCommandBuilder,
    config: &FlokiConfig,
    dind: &mut Dind,
) -> Result<DockerCommandBuilder, Error> {
    if config.dind {
        Ok(command::enable_docker_in_docker(cmd, dind)?)
    } else {
        Ok(cmd)
    }
}

fn configure_floki_user_env(cmd: DockerCommandBuilder, env: &Environment) -> DockerCommandBuilder {
    let (ref user, ref group) = env.user_details;
    let new_cmd = cmd.add_environment("FLOKI_HOST_UID", &user);
    new_cmd.add_environment("FLOKI_HOST_GID", &group)
}

fn configure_floki_host_mountdir_env(
    cmd: DockerCommandBuilder,
    floki_root: &path::Path,
) -> DockerCommandBuilder {
    cmd.add_environment(
        "FLOKI_HOST_MOUNTDIR",
        &floki_root
            .to_str()
            .expect("failed to set FLOKI_HOST_MOUNTDIR - unable to convert floki_root to str"),
    )
}

fn configure_forward_user(
    cmd: DockerCommandBuilder,
    config: &FlokiConfig,
    env: &Environment,
) -> DockerCommandBuilder {
    if config.forward_user {
        let (ref user, ref group) = env.user_details;
        cmd.add_docker_switch(&format!("--user {}:{}", user, group))
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
            Ok(command::enable_forward_ssh_agent(cmd, path)?)
        } else {
            Err(errors::FlokiError::NoSshAuthSock {})?
        }
    } else {
        Ok(cmd)
    }
}

fn configure_docker_switches(
    cmd: DockerCommandBuilder,
    config: &FlokiConfig,
) -> DockerCommandBuilder {
    let mut cmd = cmd;
    for switch in &config.docker_switches {
        cmd = cmd.add_docker_switch(&switch);
    }

    cmd
}

fn configure_working_directory(
    cmd: DockerCommandBuilder,
    env: &Environment,
    floki_root: &path::Path,
    config: &FlokiConfig,
) -> DockerCommandBuilder {
    cmd.set_working_directory(
        get_working_directory(
            &env.current_directory,
            &floki_root,
            &path::PathBuf::from(&config.mount),
        )
        .to_str()
        .unwrap(),
    )
}

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

fn get_mount_specification<'a, 'b>(
    floki_root: &'a path::Path,
    config: &'b FlokiConfig,
) -> (&'a str, &'b str) {
    (&floki_root.to_str().unwrap(), &config.mount)
}

fn build_basic_command(
    floki_root: &path::Path,
    config: &FlokiConfig,
) -> Result<(DockerCommandBuilder, Dind), Error> {
    let mount = get_mount_specification(&floki_root, &config);

    // Assign a container for docker-in-docker - we don't spawn it yet
    let dind = Dind::new(mount);

    let image = &config.image.name()?;
    let outer_shell = config.shell.outer_shell();
    let cmd = command::DockerCommandBuilder::new(image, outer_shell).add_volume(mount);

    Ok((cmd, dind))
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
}
