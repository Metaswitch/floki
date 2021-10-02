use crate::config::{DindConfig, FlokiConfig};
use crate::environment::Environment;
use crate::errors;

use failure::Error;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path;

pub(crate) struct Docker {
    pub(crate) image: String,
}

pub(crate) struct User {
    pub(crate) forward: bool,
    pub(crate) uid: nix::unistd::Uid,
    pub(crate) gid: nix::unistd::Gid,
}

pub(crate) struct SshAgent {
    pub(crate) path: OsString,
}

pub(crate) struct Paths {
    pub(crate) current_directory: path::PathBuf,
    pub(crate) root: path::PathBuf,
    pub(crate) config: path::PathBuf,
    pub(crate) workspace: path::PathBuf,
}

/// A fully resolved specification of a container to run
pub(crate) struct FlokiEnvironment {
    /// Details of the image to use
    pub(crate) image: crate::image::Image,
    /// Commands to run on initialization
    pub(crate) init: Vec<String>,
    /// Shell to use in the environment
    pub(crate) shell: crate::config::Shell,
    /// Where to mount the working directory
    pub(crate) mount: path::PathBuf,
    /// Entrypoint
    pub(crate) entrypoint: Option<String>,
    /// Volumes to mount into the container
    pub(crate) volumes: BTreeMap<String, crate::config::Volume>,
    /// User details and forwarding
    pub(crate) user: User,
    /// SSH agent forwarding
    pub(crate) ssh: Option<SshAgent>,
    /// Explicit docker switches to use
    pub(crate) docker_switches: Vec<String>,
    /// Linked docker environments
    pub(crate) docker: Option<Docker>,
    /// Paths on the host which are relevant to running
    pub(crate) paths: Paths,
}

impl FlokiEnvironment {
    pub(crate) fn from(
        config: FlokiConfig,
        environ: Environment,
    ) -> Result<FlokiEnvironment, Error> {
        let docker = match config.dind {
            DindConfig::Toggle(true) => Some(Docker {
                image: "docker:stable-dind".to_string(),
            }),
            DindConfig::Toggle(false) => None,
            DindConfig::Image { image } => Some(Docker { image }),
        };

        let user = User {
            forward: config.forward_user,
            uid: environ.user_details.uid,
            gid: environ.user_details.gid,
        };

        let entrypoint = config.entrypoint.value().map(|v| v.to_string());

        let ssh = if config.forward_ssh_agent {
            if let Some(path) = environ.ssh_agent_socket {
                Ok(Some(SshAgent { path }))
            } else {
                Err(errors::FlokiError::NoSshAuthSock {})
            }?
        } else {
            None
        };

        let paths = Paths {
            current_directory: environ.current_directory,
            root: environ.floki_root,
            config: environ.config_file,
            workspace: environ.floki_workspace,
        };

        Ok(FlokiEnvironment {
            image: config.image,
            init: config.init,
            mount: config.mount,
            shell: config.shell,
            entrypoint,
            volumes: config.volumes,
            user,
            ssh,
            docker_switches: config.docker_switches,
            docker,
            paths,
        })
    }
}
