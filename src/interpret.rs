use config::FlokiConfig;
use environment::Environment;
use dind::Dind;
use command::DockerCommandBuilder;
use command;
use errors;

use quicli::prelude::*;


/// Build a spec for the docker container, and then run it
pub(crate) fn run_container(config: &FlokiConfig, environ: &Environment, command: &str) -> Result<()> {
    let (mut cmd, mut dind) = build_basic_command(&config, &environ);

    cmd = configure_dind(cmd, &config, &mut dind)?;
    cmd = configure_floki_user_env(cmd, &environ);
    cmd = configure_forward_user(cmd, &config, &environ);
    cmd = configure_forward_ssh_agent(cmd, &config, &environ)?;
    cmd = configure_docker_switches(cmd, &config);
    cmd = configure_working_directory(cmd, &config);

    cmd.run(command)
}


pub(crate) fn command_in_shell(shell: &str, command: &Vec<String>) -> String {
    // Make sure our command runs in a subshell (we might switch user)
    let inner_shell: String = shell.to_string();
    let command_string = inner_shell + " -c \"" + &command.join(" ") + "\"";
    command_string
}


fn configure_dind(cmd: DockerCommandBuilder, config: &FlokiConfig, dind: &mut Dind) -> Result<DockerCommandBuilder> {
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


fn configure_forward_user(cmd: DockerCommandBuilder, config: &FlokiConfig, env: &Environment) -> DockerCommandBuilder {
    if config.forward_user {
        let (ref user, ref group) = env.user_details;
        cmd.add_docker_switch(&format!("--user {}:{}", user, group))
    } else {
        cmd
    }
}


fn configure_forward_ssh_agent(cmd: DockerCommandBuilder, config: &FlokiConfig, env: &Environment) -> Result<DockerCommandBuilder> {
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


fn configure_docker_switches(cmd: DockerCommandBuilder, config: &FlokiConfig) -> DockerCommandBuilder {
    let mut cmd = cmd;
    for switch in &config.docker_switches {
        cmd = cmd.add_docker_switch(&switch);
    }

    cmd
}


fn configure_working_directory(cmd: DockerCommandBuilder, config: &FlokiConfig) -> DockerCommandBuilder {
    cmd.set_working_directory(&config.mount)
}


fn get_mount_specification<'a>(config: &'a FlokiConfig, env: &'a Environment) -> (&'a str, &'a str) {
    (&env.current_directory, &config.mount)
}


fn build_basic_command(config: &FlokiConfig, env: &Environment) -> (DockerCommandBuilder, Dind) {
    let mount = get_mount_specification(&config, &env);

    // Assign a container for docker-in-docker - we don't spawn it yet
    let dind = Dind::new(mount);

    let image = &config.image.name();
    let outer_shell = config.shell.outer_shell();
    let cmd = command::DockerCommandBuilder::new(image, outer_shell)
        .add_volume(mount);

    (cmd, dind)
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
