/// Error type for floki
use std::fmt;
use std::io;
use std::process::ExitStatus;

/// FlokiSubprocessExitStatus is a structure which wraps an exit status
/// with a process description so we can pretty-print it.
pub struct FlokiSubprocessExitStatus {
    pub process_description: String,
    pub exit_status: ExitStatus,
}

/// Error types for Floki
#[derive(Debug, thiserror::Error)]
pub enum FlokiError {
    #[error("No floki.yaml found in tree")]
    ProblemFindingConfigYaml {},

    #[error("Could not normalize the file path '{name}': {error:?}")]
    ProblemNormalizingFilePath { name: String, error: io::Error },

    #[error("There was a problem opening the configuration file '{name}': {error:?}")]
    ProblemOpeningConfigYaml { name: String, error: tera::Error },

    #[error("There was a problem parsing the configuration file '{name}': {error:?}")]
    ProblemParsingConfigYaml {
        name: String,
        error: serde_yaml::Error,
    },

    #[error("Running docker command failed with error: {error:?}")]
    FailedToLaunchDocker { error: io::Error },

    #[error("Failed to complete docker command with error: {error:?}")]
    FailedToCompleteDockerCommand { error: io::Error },

    #[error("Failed to pull docker image '{image}': {exit_status:?}")]
    FailedToPullImage {
        image: String,
        exit_status: FlokiSubprocessExitStatus,
    },

    #[error("Failed to build docker image '{image}': {exit_status}")]
    FailedToBuildImage {
        image: String,
        exit_status: FlokiSubprocessExitStatus,
    },

    #[error("Failed to check existence of image '{image}': {error:?}")]
    FailedToCheckForImage { image: String, error: io::Error },

    #[error("Failed to find the key '{key}' in file '{file}'")]
    FailedToFindYamlKey { key: String, file: String },

    #[error("Running container failed: {exit_status:?}")]
    RunContainerFailed {
        exit_status: FlokiSubprocessExitStatus,
    },

    #[error("Unable to forward ssh socket - cannot find SSH_AUTH_SOCK in environment - do you have an ssh agent running?")]
    NoSshAuthSock {},

    #[error("Malformed item in docker_switches: {item}")]
    MalformedDockerSwitch { item: String },
}

/// Generate a summary string for a process exiting
fn exit_code_diagnosis(exit_status: &ExitStatus) -> String {
    match exit_status.code() {
        Some(rc) => format!("exited with return code {}", rc),
        None => "terminated by a signal".to_string(),
    }
}

/// Custom debug formatter for FlokiSubprocessExitStatus
impl fmt::Debug for FlokiSubprocessExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.process_description,
            exit_code_diagnosis(&self.exit_status)
        )
    }
}

/// Custom display formatter for FlokiSubprocessExitStatus
impl fmt::Display for FlokiSubprocessExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.process_description,
            exit_code_diagnosis(&self.exit_status)
        )
    }
}

/// Internal error types for floki - these represent failed assumptions of
/// the developers, and shouldn't actually manifest.
#[derive(Debug, thiserror::Error)]
pub enum FlokiInternalError {
    #[error("An internal assertion failed '{description}'.  This is probably a bug!")]
    InternalAssertionFailed { description: String },
}

/// Errors made by floki users.
#[derive(Debug, thiserror::Error)]
pub enum FlokiUserError {
    #[error("Invalid verbosity setting of {setting:?}. Use a setting between 0 and 3 (-vvv)")]
    InvalidVerbositySetting { setting: u8 },
}
