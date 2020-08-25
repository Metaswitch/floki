/// Configuration file format for floki
use crate::errors;
use crate::image;
use failure::Error;
use quicli::prelude::*;

use std::collections::BTreeMap;
use std::fs::File;
use std::path;

/// Specify the shell(s) for floki to run.
///
/// Floki runs the commands under [`init`][init] in an
/// "outer" shell, and then drops the user into an "inner"
/// shell.
///
/// [init]: ./struct.FlokiConfig.html#structfield.init
///
/// ---
///
/// Back to:
/// - [Floki Config](./struct.FlokiConfig.html#structfield.shell)
///
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Shell {
    /// Provide a string to specify both shells as the
    /// same.
    ///
    /// e.g.
    ///
    /// ```yaml
    /// bash
    /// ```
    ///
    Shell(String),

    /// Specify both shells separately.
    ///
    /// e.g.
    ///
    /// ```yaml
    /// inner: bash
    /// outer: sh
    /// ```
    ///
    TwoShell { inner: String, outer: String },
}

impl Shell {
    pub(crate) fn inner_shell(&self) -> &str {
        match self {
            Shell::Shell(s) => s,
            Shell::TwoShell { inner: s, outer: _ } => s,
        }
    }

    pub(crate) fn outer_shell(&self) -> &str {
        match self {
            Shell::Shell(s) => s,
            Shell::TwoShell { inner: _, outer: s } => s,
        }
    }
}

/// Enable Docker-in-Docker inside the container that
/// floki runs.
///
/// ---
///
/// Back to:
/// - [Floki Config](./struct.FlokiConfig.html#structfield.dind)
///
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum DindConfig {
    /// Provide a boolean.
    ///
    /// If `true` is given, floki will enable Docker-in-Docker
    /// using dind image: `docker:stable-dind`.
    ///
    Toggle(bool),

    /// Enable Docker-in-Docker and specify the dind image to use.
    ///
    /// e.g.
    ///
    /// ```yaml
    /// image: docker:19.03-dind
    /// ```
    ///
    Image { image: String },
}

impl DindConfig {
    pub fn deactivated() -> Self {
        DindConfig::Toggle(false)
    }

    pub fn enabled(&self) -> bool {
        match self {
            DindConfig::Toggle(v) => *v,
            _ => true,
        }
    }

    pub fn image(&self) -> &str {
        match self {
            DindConfig::Image { image } => image,
            _ => "docker:stable-dind",
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// # Floki Volumes
///
/// Floki has the ability to use volumes for caching build artifacts between
/// runs of the container (amongst other things).
///
/// Floki creates directories on the host to back these volumes
/// in `~/.floki/volumes`.
///
/// ---
///
/// Back to:
/// - [Floki Config](./struct.FlokiConfig.html#structfield.volumes)
///
pub(crate) struct Volume {
    #[serde(default = "default_to_false")]
    /// _Optional_
    ///
    /// _Default:_ `false`
    ///
    /// Share this volume with other containers instantiated from
    /// different floki config files.
    ///
    /// If `false`, this volume is only accessible to containers
    /// instantiated using this config file.
    ///
    pub(crate) shared: bool,

    /// The path to which the volume is mounted inside the container.
    ///
    pub(crate) mount: path::PathBuf,
}

/// # Floki Configuration Reference
///
/// By default floki looks for its configuration file in `floki.yaml`
/// (See `floki --help` for how to override this).
///
/// This document serves as a complete reference for all that can be
/// included in this configuration file.  See the [user documentation][ud]
/// for installation instructions, usage guidance, and recipes for
/// a number of use cases.
///
/// [ud]: https://metaswitch.github.io/floki/
///
/// Floki config is defined as a [YAML][yaml] document with the structure
/// detailed below.
///
/// [yaml]: https://yaml.org/spec/1.2/spec.html
///
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FlokiConfig {
    /// Specify how floki sources or builds the image to host your
    /// environment.
    ///
    pub(crate) image: image::Image,

    /// _Optional_
    ///
    /// Commands to be run before dropping the user into an interactive
    /// shell.
    ///
    /// If "inner" and "outer" shells are defined under [`shell`](#structfield.shell),
    /// these commands run in the outer shell.
    ///
    #[serde(default = "Vec::new")]
    pub(crate) init: Vec<String>,

    /// _Optional_
    ///
    /// _Default:_ `sh`
    ///
    /// Specify the shell for floki to run.
    ///
    #[serde(default = "default_shell")]
    pub(crate) shell: Shell,

    /// _Optional_
    ///
    /// _Default:_ `/src`
    ///
    /// Path inside the container that floki will mount the
    /// current working directory to.
    ///
    #[serde(default = "default_mount")]
    pub(crate) mount: path::PathBuf,

    /// _Optional_
    ///
    /// Extra command line arguments to pass to docker.
    ///
    /// NOTE: This is a back door and if you find you have repeated use
    /// of the same invocation using `docker_switches`, consider raising
    /// an issue to request the use case be covered with mainline
    /// floki features.
    ///
    #[serde(default = "Vec::new")]
    pub(crate) docker_switches: Vec<String>,

    /// _Optional_
    ///
    /// _Default:_ `false`
    ///
    /// Forward your ssh-agent into the container.
    ///
    /// NOTE: You will need to have an ssh-agent running on the host
    /// before launching floki.
    ///
    #[serde(default = "default_to_false")]
    pub(crate) forward_ssh_agent: bool,

    /// _Optional_
    ///
    /// _Default:_ `false`
    ///
    /// Enable Docker-in-Docker inside the container.
    ///
    #[serde(default = "DindConfig::deactivated")]
    pub(crate) dind: DindConfig,

    /// _Optional_
    ///
    /// _Default:_ `false`
    ///
    /// Run interactive shell in the container as the host user.
    ///
    #[serde(default = "default_to_false")]
    pub(crate) forward_user: bool,

    /// _Optional_
    ///
    /// Specify the volumes to mount in the container as a mapping
    /// of a name to [volume config][vol].
    ///
    /// [vol]: ./struct.Volume.html
    ///
    #[serde(default = "BTreeMap::new")]
    pub(crate) volumes: BTreeMap<String, Volume>,
}

impl FlokiConfig {
    pub fn from_file(file: &path::Path) -> Result<FlokiConfig, Error> {
        let f = File::open(file).map_err(|e| errors::FlokiError::ProblemOpeningConfigYaml {
            name: file.display().to_string(),
            error: e,
        })?;

        let mut config: FlokiConfig = serde_yaml::from_reader(f).map_err(|e| {
            errors::FlokiError::ProblemParsingConfigYaml {
                name: file.display().to_string(),
                error: e,
            }
        })?;

        // Ensure the path to an external yaml file is correct.
        // If the image.yaml.path file is relative, then it should
        // be relative to the floki config file. At this point we
        // already have the path to the floki config file, so we
        // just prepend that to image.yaml.path.
        if let image::Image::Yaml { ref mut yaml } = config.image {
            if yaml.file.is_relative() {
                yaml.file = file
                    .parent()
                    .ok_or_else(|| errors::FlokiInternalError::InternalAssertionFailed {
                        description: format!(
                            "could not constuct path to external yaml file '{:?}'",
                            &yaml.file
                        ),
                    })?
                    .join(yaml.file.clone());
            }
        }

        debug!(
            "Parsed '{}' into configuration: {:?}",
            file.display(),
            &config
        );

        Ok(config)
    }
}

fn default_shell() -> Shell {
    Shell::Shell("sh".into())
}

fn default_mount() -> path::PathBuf {
    path::Path::new("/src").to_path_buf()
}

fn default_to_false() -> bool {
    false
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestShellConfig {
        shell: Shell,
    }

    #[test]
    fn test_single_shell_config() {
        let yaml = "shell: bash";
        let expected = TestShellConfig {
            shell: Shell::Shell("bash".into()),
        };
        let actual: TestShellConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(actual == expected);
    }

    #[test]
    fn test_two_shell_config() {
        let yaml = "shell:\n  outer: sh\n  inner: bash";
        let expected_shell = Shell::TwoShell {
            inner: "bash".into(),
            outer: "sh".into(),
        };
        let expected = TestShellConfig {
            shell: expected_shell,
        };
        let actual: TestShellConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(actual == expected);
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestDindConfig {
        dind: DindConfig,
    }

    #[test]
    fn test_dind_enabled_config() {
        let yaml = "dind: true";
        let expected = TestDindConfig {
            dind: DindConfig::Toggle(true),
        };
        let actual: TestDindConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.dind.enabled(), true);
        assert_eq!(actual.dind.image(), "docker:stable-dind");
    }

    #[test]
    fn test_dind_image_config() {
        let yaml = "dind:\n  image: dind:custom";
        let expected = TestDindConfig {
            dind: DindConfig::Image {
                image: "dind:custom".into(),
            },
        };
        let actual: TestDindConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.dind.enabled(), true);
        assert_eq!(actual.dind.image(), "dind:custom");
    }
}
