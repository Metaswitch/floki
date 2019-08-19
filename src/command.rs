use crate::errors::{FlokiError, FlokiSubprocessExitStatus};
use failure::Error;
use std::path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct DockerCommandBuilder {
    volumes: Vec<(String, String)>,
    environment: Vec<(String, String)>,
    shell: String,
    switches: Vec<String>,
    image: String,
}

impl DockerCommandBuilder {
    pub fn run(&self, subshell_command: &str) -> Result<(), Error> {
        debug!(
            "Spawning docker command with configuration: {:?} args: {}",
            self, &subshell_command
        );
        let mut command = Command::new("docker")
            .args(&["run", "--rm", "-it"])
            .args(&self.build_volume_switches())
            .args(&self.build_environment_switches())
            .args(&self.build_docker_switches())
            .arg(&self.image)
            .arg(&self.shell)
            .arg("-c")
            .arg(subshell_command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .spawn()
            .map_err(|e| FlokiError::FailedToLaunchDocker { error: e })?;

        let exit_status = command
            .wait()
            .map_err(|e| FlokiError::FailedToCompleteDockerCommand { error: e })?;
        if exit_status.success() {
            Ok(())
        } else {
            Err(FlokiError::RunContainerFailed {
                exit_status: FlokiSubprocessExitStatus {
                    process_description: "docker run".into(),
                    exit_status: exit_status,
                },
            })?
        }
    }

    pub fn new(image: &str, shell: &str) -> Self {
        DockerCommandBuilder {
            volumes: Vec::new(),
            environment: Vec::new(),
            shell: shell.into(),
            switches: Vec::new(),
            image: image.into(),
        }
    }

    pub fn add_volume(mut self, spec: (&str, &str)) -> Self {
        let (src, dst) = spec;
        self.volumes.push((src.to_string(), dst.to_string()));
        self
    }

    pub fn add_environment(mut self, var: &str, bind: &str) -> Self {
        self.environment.push((var.to_string(), bind.to_string()));
        self
    }

    pub fn add_docker_switch(mut self, switch: &str) -> Self {
        self.switches.push(switch.into());
        self
    }

    pub fn set_working_directory(self, directory: &str) -> Self {
        let mut cmd = self;
        cmd = cmd.add_docker_switch("-w");
        cmd = cmd.add_docker_switch(directory);
        cmd
    }

    fn build_volume_switches(&self) -> Vec<String> {
        let mut switches = Vec::new();
        for (s, d) in self.volumes.iter() {
            switches.push("-v".into());
            switches.push(format!("{}:{}", s, d));
        }
        switches
    }

    fn build_environment_switches(&self) -> Vec<String> {
        let mut switches = Vec::new();
        for (var, bind) in self.environment.iter() {
            switches.push("-e".into());
            switches.push(format!("{}={}", var, bind));
        }
        switches
    }

    fn build_docker_switches(&self) -> Vec<String> {
        let mut switches = Vec::new();
        for docker_switch in self.switches.iter() {
            let pieces = docker_switch.split_whitespace();
            for s in pieces {
                switches.push(s.into());
            }
        }
        switches
    }
}

pub fn enable_forward_ssh_agent(
    command: DockerCommandBuilder,
    agent_socket: &str,
) -> Result<DockerCommandBuilder, Error> {
    debug!("Got SSH_AUTH_SOCK={}", agent_socket);
    if let Some(dir) = path::Path::new(&agent_socket).to_str() {
        Ok(command
            .add_environment("SSH_AUTH_SOCK", agent_socket)
            .add_volume((dir, dir)))
    } else {
        Err(FlokiError::NoSshAuthSock {})?
    }
}

pub fn enable_docker_in_docker(
    command: DockerCommandBuilder,
    dind: &mut crate::dind::Dind,
) -> Result<DockerCommandBuilder, Error> {
    debug!("docker-in-docker: {:?}", &dind);
    crate::dind::dind_preflight()?;
    dind.launch()?;
    Ok(command
        .add_docker_switch(&format!("--link {}:floki-docker", dind.name))
        .add_environment("DOCKER_HOST", "tcp://floki-docker:2375"))
}

/// Turn the init section of a floki.yaml file into a command
/// that can be given to a shell
pub(crate) fn subshell_command(init: &Vec<String>, command: String) -> String {
    let mut args = init.clone();
    args.push(command);
    args.join(" && ")
}
