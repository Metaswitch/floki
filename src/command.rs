use crate::errors::{FlokiError, FlokiSubprocessExitStatus};
use failure::Error;
use std::ffi::OsString;
use std::path;
use std::process::{Command, Stdio};
use uuid;

#[derive(Debug, Clone)]
pub struct DockerCommandBuilder {
    name: String,
    volumes: Vec<(String, String)>,
    environment: Vec<(String, String)>,
    switches: Vec<OsString>,
    image: String,
}

#[derive(Debug)]
pub struct DaemonHandle {
    name: String,
}

impl DaemonHandle {
    fn from_builder(builder: DockerCommandBuilder) -> Self {
        DaemonHandle { name: builder.name }
    }
}

impl Drop for DaemonHandle {
    fn drop(&mut self) {
        info!("Stopping daemon docker container '{}'", self.name);
        Command::new("docker")
            .args(&["kill", &self.name])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Unable to kill docker container")
            .wait()
            .expect("Unable to wait for docker container to die");
    }
}

impl DockerCommandBuilder {
    pub fn run(&self, command: &[&str]) -> Result<(), Error> {
        debug!(
            "Spawning docker command with configuration: {:?} args: {:?}",
            self, command
        );

        let mut command = Command::new("docker")
            .args(&["run", "--rm", "-it"])
            .args(&self.build_volume_switches())
            .args(&self.build_environment_switches())
            .args(self.build_docker_switches())
            .arg(&self.image)
            .args(command)
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

    pub fn start_as_daemon(self, command: &[&str]) -> Result<DaemonHandle, Error> {
        debug!("Starting daemon container '{}'", self.name);
        let exit_status = Command::new("docker")
            .args(&["run", "--rm"])
            .args(&["--name", &self.name])
            .args(&self.build_volume_switches())
            .args(&self.build_environment_switches())
            .args(self.build_docker_switches())
            .arg("-d")
            .arg(&self.image)
            .args(command)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| FlokiError::FailedToLaunchDocker { error: e })?
            .wait()
            .map_err(|e| FlokiError::FailedToCompleteDockerCommand { error: e })?;

        if exit_status.success() {
            Ok(DaemonHandle::from_builder(self))
        } else {
            Err(FlokiError::RunContainerFailed {
                exit_status: FlokiSubprocessExitStatus {
                    process_description: "docker run".into(),
                    exit_status: exit_status,
                },
            })?
        }
    }

    pub fn new(image: &str) -> Self {
        DockerCommandBuilder {
            name: uuid::Uuid::new_v4().to_string(),
            volumes: Vec::new(),
            environment: Vec::new(),
            switches: Vec::new(),
            image: image.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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
        for s in switch.split_whitespace() {
            self.switches.push(s.into());
        }
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

    fn build_docker_switches(&self) -> &Vec<OsString> {
        &self.switches
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
    dind: &crate::dind::Dind,
) -> Result<DockerCommandBuilder, Error> {
    Ok(command
        .add_docker_switch(&format!("--link {}:floki-docker", dind.name()))
        .add_environment("DOCKER_HOST", "tcp://floki-docker:2375"))
}
