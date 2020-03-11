/// Docker-in-docker structures
use failure::Error;
use std::path;

use crate::command::{DaemonHandle, DockerCommandBuilder};
use crate::image::{image_exists_locally, pull_image};

#[derive(Debug)]
pub struct Dind {
    command: DockerCommandBuilder,
}

impl Dind {
    pub fn new(image: &str, mount: (&path::PathBuf, &path::PathBuf)) -> Self {
        Dind {
            command: DockerCommandBuilder::new(image)
                .add_docker_switch("--privileged")
                .add_volume(mount),
        }
    }

    pub fn name(&self) -> &str {
        self.command.name()
    }

    pub fn launch(self) -> Result<DaemonHandle, Error> {
        info!(
            "Starting docker:dind container with name {}",
            self.command.name()
        );
        let handle = self
            .command
            .start_as_daemon(&["dockerd", "--host=tcp://0.0.0.0:2375"])?;
        info!("docker:dind launched");
        Ok(handle)
    }
}

/// Check the docker dind image is available
pub fn dind_preflight(image: &str) -> Result<(), Error> {
    if image_exists_locally(image)? {
        Ok(())
    } else {
        pull_image(image)
    }
}
