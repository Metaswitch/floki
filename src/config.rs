/// Configuration file format for floki
use crate::errors::FlokiError;
use crate::image;
use serde::{Deserialize, Serialize};
use tera::from_value;
use tera::Context;
use tera::Tera;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Shell {
    Shell(String),
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum DindConfig {
    Toggle(bool),
    Image { image: String },
}

impl DindConfig {
    pub fn deactivated() -> Self {
        DindConfig::Toggle(false)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// The Volume structure captures configuration for floki volumes
pub(crate) struct Volume {
    #[serde(default = "default_to_false")]
    /// A shared volume is reused by containers which also use a
    /// shared volume by the same name. Volumes which are not
    /// shared are localised to a particular floki configuration file.
    pub(crate) shared: bool,
    /// The mount path is the path at which the volume is mounted
    /// inside the floki container.
    pub(crate) mount: PathBuf,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Entrypoint {
    Suppress { suppress: bool },
}

impl Entrypoint {
    pub fn value(&self) -> Option<&str> {
        match self {
            Entrypoint::Suppress { suppress } if *suppress => Some(""),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FlokiConfig {
    pub(crate) image: image::Image,
    #[serde(default = "Vec::new")]
    pub(crate) init: Vec<String>,
    #[serde(default = "default_shell")]
    pub(crate) shell: Shell,
    #[serde(default = "default_mount")]
    pub(crate) mount: PathBuf,
    #[serde(default = "Vec::new")]
    pub(crate) docker_switches: Vec<String>,
    #[serde(default = "default_to_false")]
    pub(crate) forward_ssh_agent: bool,
    #[serde(default = "DindConfig::deactivated")]
    pub(crate) dind: DindConfig,
    #[serde(default = "default_to_false")]
    pub(crate) forward_user: bool,
    #[serde(default = "BTreeMap::new")]
    pub(crate) volumes: BTreeMap<String, Volume>,
    #[serde(default = "default_entrypoint")]
    pub(crate) entrypoint: Entrypoint,
}

fn path_from_args(args: &HashMap<String, tera::Value>) -> tera::Result<String> {
    let file = match args.get("file") {
        Some(file) => file,
        None => return Err("file parameter is required".into()),
    };
    Ok(from_value::<String>(file.clone())?)
}

fn yamlloader(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let path = path_from_args(args)?;
    let f = std::fs::File::open(path)?;
    serde_yaml::from_reader(f).map_err(|_| "Failed to read file".into())
}

fn jsonloader(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let path = path_from_args(args)?;
    let f = std::fs::File::open(path)?;
    serde_json::from_reader(f).map_err(|_| "Failed to read file".into())
}

fn tomlloader(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let path = path_from_args(args)?;
    let contents = std::fs::read_to_string(path)?;
    toml::from_str(&contents).map_err(|_| "Failed to read file".into())
}

// Renders a template from a given string.
pub fn render_template(template: &str, source_filename: &Path) -> Result<String, FlokiError> {
    let template_path = source_filename.display().to_string();

    debug!("Rendering template: {template_path}");

    // Read the template using tera
    let mut tera = Tera::default();

    // Allow templates to load variables files as Values.
    tera.register_function("yaml", yamlloader);
    tera.register_function("json", jsonloader);
    tera.register_function("toml", tomlloader);

    tera.add_raw_template(&template_path, template)
        .map_err(|e| FlokiError::ProblemRenderingTemplate {
            name: template_path.clone(),
            error: e,
        })?;

    // Read the environment variables and store them in a tera context
    // under the `env` name.
    let vars: HashMap<String, String> = std::env::vars().collect();
    let mut context = Context::new();
    context.insert("env", &vars);

    // Render the floki file to string using the context.
    tera.render(&template_path, &context)
        .map_err(|e| FlokiError::ProblemRenderingTemplate {
            name: template_path.clone(),
            error: e,
        })
}

impl FlokiConfig {
    pub fn render(file: &Path) -> Result<String, FlokiError> {
        let content =
            std::fs::read_to_string(file).map_err(|e| FlokiError::ProblemOpeningConfigYaml {
                name: file.display().to_string(),
                error: e,
            })?;

        // Render the template first before parsing it.
        render_template(&content, file)
    }

    pub fn from_file(file: &Path) -> Result<Self, FlokiError> {
        debug!("Reading configuration file: {:?}", file);

        // Render the output from the configuration file before parsing.
        let output = Self::render(file)?;

        // Parse the rendered floki file from the string.
        let mut config: FlokiConfig =
            serde_yaml::from_str(&output).map_err(|e| FlokiError::ProblemParsingConfigYaml {
                name: file.display().to_string(),
                error: e,
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
                    .ok_or_else(|| FlokiError::InternalAssertionFailed {
                        description: format!(
                            "could not construct path to external yaml file '{:?}'",
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

fn default_mount() -> PathBuf {
    Path::new("/src").to_path_buf()
}

fn default_to_false() -> bool {
    false
}

fn default_entrypoint() -> Entrypoint {
    Entrypoint::Suppress { suppress: true }
}

#[cfg(test)]
mod test {
    use super::*;

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
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestEntrypointConfig {
        entrypoint: Entrypoint,
    }

    #[test]
    fn test_entrypoint_suppress() {
        let yaml = "entrypoint:\n  suppress: true";
        let expected = TestEntrypointConfig {
            entrypoint: Entrypoint::Suppress { suppress: true },
        };
        let actual: TestEntrypointConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.entrypoint.value(), Some(""));
    }

    #[test]
    fn test_entrypoint_no_suppress() {
        let yaml = "entrypoint:\n  suppress: false";
        let expected = TestEntrypointConfig {
            entrypoint: Entrypoint::Suppress { suppress: false },
        };
        let actual: TestEntrypointConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.entrypoint.value(), None);
    }

    #[test]
    fn test_tera_render() -> Result<(), Box<dyn std::error::Error>> {
        let template = r#"{% set var = "test" %}image: {{ var }}"#;
        let config = render_template(template, Path::new("floki"))?;
        assert_eq!(config, "image: test");
        Ok(())
    }

    #[test]
    fn test_tera_yamlload() -> Result<(), Box<dyn std::error::Error>> {
        let template =
            r#"{% set values = yaml(file="test_resources/values.yaml") %}shell: {{ values.foo }}"#;
        let config = render_template(template, Path::new("floki"))?;
        assert_eq!(config, "shell: bar");
        Ok(())
    }

    #[test]
    fn test_tera_jsonload() -> Result<(), Box<dyn std::error::Error>> {
        let template =
            r#"{% set values = json(file="test_resources/values.json") %}shell: {{ values.foo }}"#;
        let config = render_template(template, Path::new("floki"))?;
        assert_eq!(config, "shell: bar");
        Ok(())
    }

    #[test]
    fn test_tera_tomlload() -> Result<(), Box<dyn std::error::Error>> {
        let template =
            r#"{% set values = toml(file="Cargo.toml") %}floki: {{ values.package.name }}"#;
        let config = render_template(template, Path::new("floki"))?;
        assert_eq!(config, "floki: floki");
        Ok(())
    }
}
