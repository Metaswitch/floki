use crate::errors::{FlokiError, FlokiSubprocessExitStatus};
use anyhow::Error;
use std::ffi::{OsStr, OsString};
use std::path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct DockerCommandBuilder {
    name: String,
    volumes: Vec<OsString>,
    environment: Vec<OsString>,
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
            .args(self.base_args())
            .args(&self.build_volume_switches())
            .args(self.build_environment_switches())
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
                    exit_status,
                },
            }
            .into())
        }
    }

    pub fn start_as_daemon(self, command: &[&str]) -> Result<DaemonHandle, Error> {
        debug!("Starting daemon container '{}'", self.name);
        let exit_status = Command::new("docker")
            .args(&["run", "--rm"])
            .args(&["--name", &self.name])
            .args(&self.build_volume_switches())
            .args(self.build_environment_switches())
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
                    exit_status,
                },
            }
            .into())
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

    pub fn add_volume(mut self, spec: (&path::PathBuf, &path::PathBuf)) -> Self {
        let (src, dst) = spec;
        self.volumes.push(Self::volume_mapping(src, dst));
        self
    }

    pub fn add_environment<V: AsRef<OsStr>, B: AsRef<OsStr>>(mut self, var: V, bind: B) -> Self {
        self.environment.push("-e".into());
        self.environment.push(Self::environment_mapping(var, bind));
        self
    }

    pub fn add_docker_switch<S: AsRef<OsStr>>(mut self, switch: S) -> Self {
        self.switches.push(switch.as_ref().into());
        self
    }

    pub fn set_working_directory<S: AsRef<OsStr>>(self, directory: S) -> Self {
        let mut cmd = self;
        cmd = cmd.add_docker_switch("-w");
        cmd = cmd.add_docker_switch(directory);
        cmd
    }

    fn build_volume_switches(&self) -> Vec<&OsStr> {
        let mut switches = Vec::new();
        for mapping in self.volumes.iter() {
            switches.push("-v".as_ref());
            switches.push(mapping.as_os_str());
        }
        switches
    }

    fn volume_mapping(src: &path::Path, dst: &path::Path) -> OsString {
        let mut mapping = src.to_path_buf().into_os_string();
        mapping.push(":");
        mapping.push(dst);
        mapping
    }

    fn environment_mapping<V: AsRef<OsStr>, B: AsRef<OsStr>>(var: V, bind: B) -> OsString {
        let mut binding: OsString = var.as_ref().into();
        binding.push("=");
        binding.push(bind);
        binding
    }

    fn build_environment_switches(&self) -> &Vec<OsString> {
        &self.environment
    }

    fn build_docker_switches(&self) -> &Vec<OsString> {
        &self.switches
    }

    fn base_args(&self) -> Vec<&OsStr> {
        let mut base_args: Vec<&OsStr> = vec!["run".as_ref(), "--rm".as_ref(), "-t".as_ref()];
        if atty::is(atty::Stream::Stdout) {
            base_args.push("-i".as_ref());
        }
        base_args
    }
}

pub fn enable_forward_ssh_agent(
    command: DockerCommandBuilder,
    agent_socket: &OsStr,
) -> DockerCommandBuilder {
    debug!("Got SSH_AUTH_SOCK={:?}", agent_socket);
    let dir = path::Path::new(agent_socket).to_path_buf();
    command
        .add_environment("SSH_AUTH_SOCK", agent_socket)
        .add_volume((&dir, &dir))
}

pub fn enable_docker_in_docker(
    command: DockerCommandBuilder,
    dind: &crate::dind::Dind,
) -> Result<DockerCommandBuilder, Error> {
    Ok(command
        .add_docker_switch("--link")
        .add_docker_switch(&format!("{}:floki-docker", dind.name()))
        .add_environment("DOCKER_HOST", "tcp://floki-docker:2375"))
}
