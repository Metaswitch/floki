use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use yaml_rust::YamlLoader;

use crate::errors::{FlokiError, FlokiSubprocessExitStatus};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildSpec {
    name: String,
    #[serde(default = "default_dockerfile")]
    dockerfile: PathBuf,
    #[serde(default = "default_context")]
    context: PathBuf,
    target: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct YamlSpec {
    pub file: PathBuf,
    key: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecSpec {
    command: String,
    args: Vec<String>,
    image: String,
}

fn default_dockerfile() -> PathBuf {
    "Dockerfile".into()
}

fn default_context() -> PathBuf {
    ".".into()
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Image {
    Name(String),
    Build { build: BuildSpec },
    Yaml { yaml: YamlSpec },
    Exec { exec: ExecSpec },
}

impl Image {
    /// Name of the image
    pub fn name(&self) -> Result<String, Error> {
        match *self {
            Image::Name(ref s) => Ok(s.clone()),
            Image::Build { ref build } => Ok(build.name.clone() + ":floki"),
            Image::Yaml { ref yaml } => {
                let contents = fs::read_to_string(&yaml.file)?;
                let raw = YamlLoader::load_from_str(&contents)?;
                let path = yaml.key.split('.').collect::<Vec<_>>();
                let mut val = &raw[0];

                for key in &path {
                    // Yaml arrays and maps with scalar keys can both be indexed by
                    // usize, so heuristically prefer a usize index to a &str index.
                    val = match key.parse::<usize>() {
                        Ok(x) => &val[x],
                        Err(_) => &val[*key],
                    };
                }
                val.as_str()
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| {
                        FlokiError::FailedToFindYamlKey {
                            key: yaml.key.to_string(),
                            file: yaml.file.display().to_string(),
                        }
                        .into()
                    })
            }
            Image::Exec { ref exec } => Ok(exec.image.clone()),
        }
    }

    /// Do the required work to get the image, and then return
    /// it's name
    pub fn obtain_image(&self, floki_root: &Path) -> Result<String, Error> {
        match *self {
            // Deal with the case where want to build an image
            Image::Build { ref build } => {
                let mut command = Command::new("docker");
                command
                    .arg("build")
                    .arg("-t")
                    .arg(self.name()?)
                    .arg("-f")
                    .arg(&floki_root.join(&build.dockerfile));

                if let Some(target) = &build.target {
                    command.arg("--target").arg(target);
                }

                let exit_status = command
                    .arg(&floki_root.join(&build.context))
                    .spawn()?
                    .wait()?;
                if exit_status.success() {
                    Ok(self.name()?)
                } else {
                    Err(FlokiError::FailedToBuildImage {
                        image: self.name()?,
                        exit_status: FlokiSubprocessExitStatus {
                            process_description: "docker build".into(),
                            exit_status,
                        },
                    }
                    .into())
                }
            }
            Image::Exec { ref exec } => {
                let exit_status = Command::new(&exec.command)
                    .args(&exec.args)
                    .spawn()?
                    .wait()?;

                if exit_status.success() {
                    Ok(self.name()?)
                } else {
                    Err(FlokiError::FailedToBuildImage {
                        image: self.name()?,
                        exit_status: FlokiSubprocessExitStatus {
                            process_description: exec.command.clone(),
                            exit_status,
                        },
                    }
                    .into())
                }
            }
            // All other cases we just return the name
            _ => Ok(self.name()?),
        }
    }
}

// Now we have some functions which are useful in general

/// Wrapper to pull an image by it's name
pub fn pull_image(name: &str) -> Result<(), Error> {
    debug!("Pulling image: {}", name);
    let exit_status = Command::new("docker")
        .arg("pull")
        .arg(name)
        .spawn()?
        .wait()?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(FlokiError::FailedToPullImage {
            image: name.into(),
            exit_status: FlokiSubprocessExitStatus {
                process_description: "docker pull".into(),
                exit_status,
            },
        }
        .into())
    }
}

/// Determine whether an image exists locally
pub fn image_exists_locally(name: &str) -> Result<bool, Error> {
    let ret = Command::new("docker")
        .args(&["history", "docker:stable-dind"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| FlokiError::FailedToCheckForImage {
            image: name.to_string(),
            error: e,
        })?;

    Ok(ret.code() == Some(0))
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestImage {
        image: Image,
    }

    #[test]
    fn test_image_spec_by_string() {
        let yaml = "image: foo";
        let expected = TestImage {
            image: Image::Name("foo".into()),
        };
        let actual: TestImage = serde_yaml::from_str(yaml).unwrap();
        assert!(actual == expected);
    }

    #[test]
    fn test_image_spec_by_build_spec() {
        let yaml = "image:\n  build:\n    name: foo\n    dockerfile: Dockerfile.test \n    context: ./context\n    target: builder";
        let expected = TestImage {
            image: Image::Build {
                build: BuildSpec {
                    name: "foo".into(),
                    dockerfile: "Dockerfile.test".into(),
                    context: "./context".into(),
                    target: Some("builder".into()),
                },
            },
        };
        let actual: TestImage = serde_yaml::from_str(yaml).unwrap();
        assert!(actual == expected);
    }

    #[test]
    fn test_image_spec_by_exec_spec() {
        let yaml = r#"
image:
    exec:
        command: foo
        args:
            - build
        image: "foobuild:1.0.0"
"#;
        let expected = TestImage {
            image: Image::Exec {
                exec: ExecSpec {
                    command: "foo".into(),
                    args: vec!["build".into()],
                    image: "foobuild:1.0.0".into(),
                },
            },
        };
        let actual: TestImage = serde_yaml::from_str(yaml).unwrap();
        assert!(actual == expected);
    }
}
