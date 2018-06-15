/// Configuration file format for floki
use image;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Shell {
    Shell(String),
    TwoShell {
        inner: String,
        outer: String
    }
}

impl Shell {
    pub fn inner_shell(&self) -> &str {
        match self {
            Shell::Shell(s) => s,
            Shell::TwoShell { inner: s, outer: _ } => s
        }
    }

    pub fn outer_shell(&self) -> &str {
        match self {
            Shell::Shell(s) => s,
            Shell::TwoShell { inner: _, outer: s } => s
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct FlokiConfig {
    pub(crate) image: image::Image,
    #[serde(default = "Vec::new")]
    pub(crate) init: Vec<String>,
    #[serde(default = "default_shell")]
    pub(crate) shell: Shell,
    #[serde(default = "default_mount")]
    pub(crate) mount_pwd: String,
    #[serde(default = "Vec::new")]
    pub(crate) docker_switches: Vec<String>,
    #[serde(default = "default_to_false")]
    pub(crate) forward_ssh_agent: bool,
    #[serde(default = "default_to_false")]
    pub(crate) forward_tmux_socket: bool,
    #[serde(default = "default_to_false")]
    pub(crate) dind: bool,
    #[serde(default = "default_to_false")]
    pub(crate) forward_user: bool,
}

fn default_shell() -> Shell {
    Shell::Shell("bash".into())
}

fn default_mount() -> String {
    "/src".into()
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
        shell: Shell
    }

    #[test]
    fn test_single_shell_config() {
        let yaml = "shell: bash";
        let expected = TestShellConfig {
            shell: Shell::Shell("bash".into())
        };
        let actual: TestShellConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(actual == expected);
    }

    #[test]
    fn test_two_shell_config() {
        let yaml = "shell:\n  outer: sh\n  inner: bash";
        let expected_shell = Shell::TwoShell {
            inner: "bash".into(),
            outer: "sh".into()
        };
        let expected = TestShellConfig {
            shell: expected_shell
        };
        let actual: TestShellConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(actual == expected);
    }
}
