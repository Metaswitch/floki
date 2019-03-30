use config::FlokiConfig;
use environment::Environment;
use dind::Dind;
use command::DockerCommandBuilder;
use command;
use errors;

use quicli::prelude::*;


pub(crate) fn configure_dind(cmd: DockerCommandBuilder, config: &FlokiConfig, dind: &mut Dind) -> Result<DockerCommandBuilder> {
    if config.dind {
        Ok(command::enable_docker_in_docker(cmd, dind)?)
    } else {
        Ok(cmd)
    }
}


pub(crate) fn configure_floki_user_env(cmd: DockerCommandBuilder, env: &Environment) -> DockerCommandBuilder {
    let (ref user, ref group) = env.user_details;
    let new_cmd = cmd.add_environment("FLOKI_HOST_UID", &user);
    new_cmd.add_environment("FLOKI_HOST_GID", &group)
}


pub(crate) fn configure_forward_user(cmd: DockerCommandBuilder, config: &FlokiConfig, env: &Environment) -> DockerCommandBuilder {
    if config.forward_user {
        let (ref user, ref group) = env.user_details;
        cmd.add_docker_switch(&format!("--user {}:{}", user, group))
    } else {
        cmd
    }
}


pub(crate) fn configure_forward_ssh_agent(cmd: DockerCommandBuilder, config: &FlokiConfig, env: &Environment) -> Result<DockerCommandBuilder> {
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


pub(crate) fn configure_docker_switches(cmd: DockerCommandBuilder, config: &FlokiConfig) -> DockerCommandBuilder {
    let mut cmd = cmd;
    for switch in &config.docker_switches {
        cmd = cmd.add_docker_switch(&switch);
    }

    cmd
}


pub(crate) fn configure_working_directory(cmd: DockerCommandBuilder, config: &FlokiConfig) -> DockerCommandBuilder {
    cmd.set_working_directory(&config.mount)
}


pub(crate) fn get_mount_specification<'a>(config: &'a FlokiConfig, env: &'a Environment) -> (&'a str, &'a str) {
    (&env.current_directory, &config.mount)
}
