use crate::config::{DindConfig, FlokiConfig};
use crate::dind::DEFAULT_DIND_IMAGE;
use crate::environment::Environment;
use crate::errors;

use anyhow::Error;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path;

/// Information for running docker-in-docker
#[derive(Debug)]
pub(crate) struct Dind {
    /// The image to use
    pub(crate) image: String,
}

/// Information about the user
#[derive(Debug)]
pub(crate) struct User {
    /// Should the user be forwarded?
    pub(crate) forward: bool,
    /// User host UID
    pub(crate) uid: nix::unistd::Uid,
    /// User host GID
    pub(crate) gid: nix::unistd::Gid,
}

/// Information about the host SSH agent
#[derive(Debug)]
pub(crate) struct SshAgent {
    /// Path to the agents socket
    pub(crate) path: OsString,
}

/// Paths used for running floki
#[derive(Debug)]
pub(crate) struct Paths {
    /// The internal working directory
    pub(crate) internal_working_directory: path::PathBuf,
    /// The root directory for the project (location of floki.yaml or
    /// configuration file)
    pub(crate) root: path::PathBuf,
    /// The path to the configuration file
    pub(crate) config: path::PathBuf,
    /// The base directory for storing volumes
    pub(crate) workspace: path::PathBuf,
}

/// FlokiSpec provides a fully resolved and preprocessed block of
/// configuration data which is clearer to construct a command from.
#[derive(Debug)]
pub(crate) struct FlokiSpec {
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
    pub(crate) ssh_agent: Option<SshAgent>,
    /// Explicit docker switches to use
    pub(crate) docker_switches: Vec<String>,
    /// Linked docker environments
    pub(crate) dind: Option<Dind>,
    /// Paths on the host which are relevant to running
    pub(crate) paths: Paths,
    /// Whether we're running an interactive shell in the container vs. just a one-off command
    pub(crate) interactive: bool,
}

impl FlokiSpec {
    pub(crate) fn from(
        config: FlokiConfig,
        environ: Environment,
        interactive: bool,
    ) -> Result<Self, Error> {
        let dind = match config.dind {
            DindConfig::Toggle(true) => Some(Dind {
                image: DEFAULT_DIND_IMAGE.to_string(),
            }),
            DindConfig::Toggle(false) => None,
            DindConfig::Image { image } => Some(Dind { image }),
        };

        let user = User {
            forward: config.forward_user,
            uid: environ.user_details.uid,
            gid: environ.user_details.gid,
        };

        let entrypoint = config.entrypoint.value().map(|v| v.to_string());

        let ssh_agent = if config.forward_ssh_agent {
            if let Some(path) = environ.ssh_agent_socket {
                Ok(Some(SshAgent { path }))
            } else {
                Err(errors::FlokiError::NoSshAuthSock {})
            }?
        } else {
            None
        };

        let internal_working_directory = get_working_directory(
            &environ.current_directory,
            &environ.floki_root,
            &path::PathBuf::from(&config.mount),
        );

        let paths = Paths {
            internal_working_directory,
            root: environ.floki_root,
            config: environ.config_file,
            workspace: environ.floki_workspace,
        };

        let docker_switches = decompose_switches(&config.docker_switches)?;

        let spec = FlokiSpec {
            image: config.image,
            init: config.init,
            mount: config.mount,
            shell: config.shell,
            entrypoint,
            volumes: config.volumes,
            user,
            ssh_agent,
            docker_switches,
            dind,
            paths,
            interactive,
        };

        debug!("built spec from config and environment: {:?}", spec);

        Ok(spec)
    }
}

fn decompose_switches(specs: &[String]) -> Result<Vec<String>, Error> {
    let mut flattened = Vec::new();

    for spec in specs {
        if let Some(switches) = shlex::split(spec) {
            for s in switches {
                flattened.push(s);
            }
        } else {
            return Err(errors::FlokiError::MalformedDockerSwitch { item: spec.into() }.into());
        }
    }

    Ok(flattened)
}

/// Determine what directory we are currently in
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_decompose_switches() -> Result<(), Error> {
        let switches = vec!["-e FOO='bar baz'".to_string()];

        let want: Vec<String> = vec!["-e".to_string(), "FOO=bar baz".to_string()];

        let got = decompose_switches(&switches)?;

        assert_eq!(want, got);

        Ok(())
    }

    #[test]
    fn test_decompose_switches_error() {
        let switches = vec!["-e FOO='bar baz".to_string()];
        let got = decompose_switches(&switches);
        assert!(got.is_err());
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
