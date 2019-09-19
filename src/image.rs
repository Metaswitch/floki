use failure::Error;
use quicli::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use yaml_rust::YamlLoader;

use crate::errors::{FlokiError, FlokiSubprocessExitStatus};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildSpec {
    name: String,
    #[serde(default = "default_dockerfile")]
    dockerfile: String,
    #[serde(default = "default_context")]
    context: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct YamlSpec {
    pub file: PathBuf,
    key: String,
}

fn default_dockerfile() -> String {
    "Dockerfile".into()
}

fn default_context() -> String {
    ".".into()
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Image {
    Name(String),
    Build { build: BuildSpec },
    Yaml { yaml: YamlSpec },
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
        }
    }

    /// Do the required work to get the image, and then return
    /// it's name
    pub fn obtain_image(&self) -> Result<String, Error> {
        match *self {
            // Deal with the case where want to build an image
            Image::Build { ref build } => {
                let exit_status = Command::new("docker")
                    .arg("build")
                    .arg("-t")
                    .arg(self.name()?)
                    .arg("-f")
                    .arg(&build.dockerfile)
                    .arg(&build.context)
                    .spawn()?
                    .wait()?;
                if exit_status.success() {
                    Ok(self.name()?)
                } else {
                    Err(FlokiError::FailedToBuildImage {
                        image: self.name()?,
                        exit_status: FlokiSubprocessExitStatus {
                            process_description: "docker build".into(),
                            exit_status: exit_status,
                        },
                    })?
                }
            }
            // All other cases we just return the name
            _ => Ok(self.name()?),
        }
    }
}

// Now we have some functions which are useful in general

/// Wrapper to pull an image by it's name
pub fn pull_image(name: String) -> Result<(), Error> {
    let exit_status = Command::new("docker")
        .arg("pull")
        .arg(name.clone())
        .spawn()?
        .wait()?;

    if exit_status.success() {
        Ok(())
    } else {
        Err(FlokiError::FailedToPullImage {
            image: name.clone(),
            exit_status: FlokiSubprocessExitStatus {
                process_description: "docker pull".into(),
                exit_status: exit_status,
            },
        })?
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
        let yaml = "image:\n  build:\n    name: foo\n    dockerfile: Dockerfile.test \n    context: ./context";
        let expected = TestImage {
            image: Image::Build {
                build: BuildSpec {
                    name: "foo".into(),
                    dockerfile: "Dockerfile.test".into(),
                    context: "./context".into(),
                },
            },
        };
        let actual: TestImage = serde_yaml::from_str(yaml).unwrap();
        assert!(actual == expected);
    }
}
