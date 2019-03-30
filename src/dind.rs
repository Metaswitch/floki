/// Docker-in-docker structures
use quicli::prelude::*;
use std::process::{Command, Stdio};
use uuid;

use errors::FlokiError;
use image::{image_exists_locally, pull_image};

#[derive(Debug)]
pub struct Dind {
    pub name: String,
    // The location of the mount directory to share with the dind container.
    mount_source: String,
    mount_target: String,
    // Have we started a dind container?
    started: bool,
}

impl Dind {
    pub fn new(mount: (&str, &str)) -> Self {
        let (src, dst) = mount;
        Dind {
            name: uuid::Uuid::new_v4().to_string(),
            mount_source: src.to_string(),
            mount_target: dst.to_string(),
            started: false,
        }
    }

    pub fn launch(&mut self) -> Result<()> {
        info!("Starting docker:dind container with name {}", &self.name);
        Command::new("docker")
            .args(&[
                "run",
                "--rm",
                "--privileged",
                "--name",
                &self.name,
                "-v",
                &format!("{}:{}", self.mount_source, self.mount_target),
                "-d",
                "docker:stable-dind",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| FlokiError::FailedToLaunchDocker { error: e })?
            .wait()
            .map_err(|e| FlokiError::FailedToCompleteDockerCommand { error: e })?;

        self.started = true;

        info!("docker:dind launched");

        Ok(())
    }
}

impl Drop for Dind {
    fn drop(&mut self) {
        if self.started {
            info!("Stopping docker:dind container {}", &self.name);
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
}

/// Check the docker dind image is available
pub fn dind_preflight() -> Result<()> {
    if image_exists_locally("docker:stable-dind".into())? {
        Ok(())
    } else {
        pull_image("docker:stable-dind".into())
    }
}
