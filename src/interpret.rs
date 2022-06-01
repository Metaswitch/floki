use crate::command;
use crate::command::DockerCommandBuilder;
use crate::dind::Dind;
use crate::spec;
use crate::volumes::resolve_volume_mounts;

use anyhow::Error;
use std::path;

pub(crate) fn run_floki_container(
    spec: &spec::FlokiSpec,
    inner_command: &str,
) -> Result<(), Error> {
    spec.image.obtain_image(&spec.paths.root)?;

    let mut cmd = command::DockerCommandBuilder::new(&spec.image.name()?)
        .add_volume((&spec.paths.root, &spec.mount));

    let volumes = resolve_volume_mounts(&spec.paths.config, &spec.paths.workspace, &spec.volumes);
    instantiate_volumes(&volumes)?;

    cmd = configure_volumes(cmd, &volumes);
    cmd = cmd.add_environment("FLOKI_HOST_MOUNTDIR", &spec.paths.root);
    cmd = cmd.add_environment("FLOKI_HOST_UID", spec.user.uid.to_string());
    cmd = cmd.add_environment("FLOKI_HOST_GID", spec.user.gid.to_string());
    cmd = cmd.set_working_directory(&spec.paths.internal_working_directory);
    cmd = cmd.set_interactive(spec.interactive);

    if spec.user.forward {
        cmd = cmd
            .add_docker_switch("--user")
            .add_docker_switch(&format!("{}:{}", spec.user.uid, spec.user.gid));
    }

    if let Some(spec::SshAgent { path }) = &spec.ssh_agent {
        cmd = command::enable_forward_ssh_agent(cmd, path);
    }

    if let Some(entrypoint) = &spec.entrypoint {
        cmd = cmd.add_docker_switch(&format!("--entrypoint={}", entrypoint))
    }

    for switch in &spec.docker_switches {
        cmd = cmd.add_docker_switch(switch);
    }

    // Finally configure dind, taking care to hold a handle for the linked dind container
    let _handle = if let Some(spec::Dind { image }) = &spec.dind {
        let dind = Dind::new(image, (&spec.paths.root, &spec.mount));
        cmd = command::enable_docker_in_docker(cmd, &dind)?;
        crate::dind::dind_preflight(image)?;
        Some(dind.launch()?)
    } else {
        None
    };

    let subshell_command = subshell_command(&spec.init, inner_command);
    cmd.run(&[spec.shell.outer_shell(), "-c", &subshell_command])
}

pub(crate) fn command_in_shell(shell: &str, command: &[String]) -> String {
    // Make sure our command runs in a subshell (we might switch user)
    let inner_shell: String = shell.to_string();
    inner_shell + " -c \"" + &command.join(" ") + "\""
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

/// Turn the init section of a floki.yaml file into a command
/// that can be given to a shell
fn subshell_command(init: &[String], command: &str) -> String {
    let mut args: Vec<&str> = init.iter().map(|s| s as &str).collect::<Vec<&str>>();
    args.push(command);
    args.join(" && ")
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
}
