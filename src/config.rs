/// Configuration file format for floki
use crate::errors::FlokiError;
use crate::image;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
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

impl Default for Shell {
    fn default() -> Self {
        Self::Shell("/bin/sh".into())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum DindConfig {
    Toggle(bool),
    Image { image: String },
}

impl Default for DindConfig {
    fn default() -> Self {
        DindConfig::Toggle(false)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// The Volume structure captures configuration for floki volumes
pub(crate) struct Volume {
    #[serde(default)]
    /// A shared volume is reused by containers which also use a
    /// shared volume by the same name. Volumes which are not
    /// shared are localised to a particular floki configuration file.
    pub(crate) shared: bool,
    /// The mount path is the path at which the volume is mounted
    /// inside the floki container.
    pub(crate) mount: PathBuf,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
#[serde(untagged)]
pub(crate) enum Entrypoint {
    Suppress { suppress: bool },
}

impl Entrypoint {
    pub fn value(&self) -> Option<&str> {
        match self {
            &Entrypoint::Suppress { suppress } if suppress => Some(""),
            _ => None,
        }
    }
}

impl Default for Entrypoint {
    fn default() -> Self {
        Self::Suppress { suppress: false }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FlokiConfig {
    pub(crate) image: image::Image,
    #[serde(default)]
    pub(crate) init: Vec<String>,
    #[serde(default)]
    pub(crate) shell: Shell,
    #[serde(default = "default_mount")]
    pub(crate) mount: PathBuf,
    #[serde(default)]
    pub(crate) docker_switches: Vec<String>,
    #[serde(default)]
    pub(crate) forward_ssh_agent: bool,
    #[serde(default)]
    pub(crate) dind: DindConfig,
    #[serde(default)]
    pub(crate) forward_user: bool,
    #[serde(default)]
    pub(crate) volumes: BTreeMap<String, Volume>,
    #[serde(default)]
    pub(crate) entrypoint: Entrypoint,
}

fn default_mount() -> PathBuf {
    PathBuf::from("/src")
}

fn path_from_args(args: &HashMap<String, tera::Value>) -> tera::Result<String> {
    let file = match args.get("file") {
        Some(file) => file,
        None => return Err("file parameter is required".into()),
    };
    Ok(from_value::<String>(file.clone())?)
}

enum LoaderType {
    Yaml,
    Json,
    Toml,
}

fn strip_yaml_tags(value: &YamlValue) -> YamlValue {
    match value {
        // For tagged values, we just return the inner value.
        YamlValue::Tagged(tagged) => strip_yaml_tags(&tagged.value),

        // For sequences and mappings, we need to recursively strip tags.
        YamlValue::Sequence(items) => {
            let mut stripped = Vec::with_capacity(items.len());
            for item in items {
                stripped.push(strip_yaml_tags(item));
            }
            YamlValue::Sequence(stripped)
        }
        YamlValue::Mapping(map) => {
            let mut stripped = YamlMapping::with_capacity(map.len());
            for (key, value) in map {
                stripped.insert(strip_yaml_tags(key), strip_yaml_tags(value));
            }
            YamlValue::Mapping(stripped)
        }
        _ => value.clone(),
    }
}

fn makeloader(path: &Path, loader: LoaderType) -> impl tera::Function {
    // Get the dirname of the Path given (if a file), or just the directory.
    let directory = if path.is_file() {
        path.parent().expect("File should have a parent directory")
    } else {
        path
    }
    .to_path_buf();

    Box::new(move |args: &HashMap<String, tera::Value>| {
        path_from_args(args)
            // Calculate the full path using the parent directory
            .map(|path| directory.join(path))
            // Read the file as a string
            .and_then(|full_path| std::fs::read_to_string(full_path).map_err(Into::into))
            // Parse the file using the relevant parser
            .and_then(|contents| match loader {
                LoaderType::Yaml => {
                    let raw: YamlValue = serde_yaml::from_str(&contents).map_err(|err| {
                        tera::Error::msg(format!("Failed to parse file as YAML: {err}"))
                    })?;
                    let stripped = strip_yaml_tags(&raw);
                    serde_yaml::from_value::<tera::Value>(stripped).map_err(|err| {
                        tera::Error::msg(format!("Failed to convert YAML value: {err}"))
                    })
                }
                LoaderType::Json => serde_json::from_str(&contents)
                    .map_err(|err| format!("Failed to parse file as JSON: {err}").into()),
                LoaderType::Toml => toml::from_str(&contents)
                    .map_err(|err| format!("Failed to parse file as TOML: {err}").into()),
            })
    })
}

// Renders a template from a given string.
pub fn render_template(template: &str, source_filename: &Path) -> Result<String, FlokiError> {
    let template_path = source_filename.display().to_string();
    debug!("Rendering template: {template_path}");

    // Get the canonical path for the template.
    let canonical_path = std::fs::canonicalize(source_filename).map_err(|err| {
        FlokiError::ProblemNormalizingFilePath {
            name: template_path.clone(),
            error: err,
        }
    })?;
    debug!("Canonical path: {canonical_path:?}");

    // Read the template using tera
    let mut tera = Tera::default();

    // Allow templates to load variables files as Values.
    tera.register_function("yaml", makeloader(&canonical_path, LoaderType::Yaml));
    tera.register_function("json", makeloader(&canonical_path, LoaderType::Json));
    tera.register_function("toml", makeloader(&canonical_path, LoaderType::Toml));

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
        debug!("Reading configuration file: {file:?}");

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
        let config = render_template(template, Path::new("floki.yaml"))?;
        assert_eq!(config, "image: test");
        Ok(())
    }

    #[test]
    fn test_tera_yamlload() -> Result<(), Box<dyn std::error::Error>> {
        let template =
            r#"{% set values = yaml(file="test_resources/values.yaml") %}shell: {{ values.foo }}"#;
        let config = render_template(template, Path::new("floki.yaml"))?;
        assert_eq!(config, "shell: bar");
        Ok(())
    }

    #[test]
    fn test_tera_jsonload() -> Result<(), Box<dyn std::error::Error>> {
        let template =
            r#"{% set values = json(file="test_resources/values.json") %}shell: {{ values.foo }}"#;
        let config = render_template(template, Path::new("floki.yaml"))?;
        assert_eq!(config, "shell: bar");
        Ok(())
    }

    #[test]
    fn test_tera_tomlload() -> Result<(), Box<dyn std::error::Error>> {
        let template =
            r#"{% set values = toml(file="Cargo.toml") %}floki: {{ values.package.name }}"#;
        let config = render_template(template, Path::new("floki.yaml"))?;
        assert_eq!(config, "floki: floki");
        Ok(())
    }

    #[test]
    fn test_strip_yaml_tags_drops_reference_tag() {
        let yaml = "value: !reference [template, script]";
        let raw: YamlValue = serde_yaml::from_str(yaml).unwrap();
        let stripped = strip_yaml_tags(&raw);

        let map = match stripped {
            YamlValue::Mapping(map) => map,
            other => panic!("expected mapping, got {:?}", other),
        };

        let key = YamlValue::String("value".into());
        let field = map.get(&key).expect("missing value key");

        match field {
            YamlValue::Sequence(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], YamlValue::String("template".into()));
                assert_eq!(items[1], YamlValue::String("script".into()));
            }
            other => panic!("expected sequence, got {:?}", other),
        }
    }

    #[test]
    fn test_tera_yamlload_with_gitlab_reference() -> Result<(), Box<dyn std::error::Error>> {
        let template = r#"{% set values = yaml(file="test_resources/gitlab_reference.yaml") %}script0: {{ values.job.script[0] }} script1: {{ values.job.script[1] }}"#;
        let rendered = render_template(template, Path::new("floki.yaml"))?;
        assert_eq!(rendered, "script0: .shared_template script1: script");
        Ok(())
    }
}
