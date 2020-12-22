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
#[derive(Debug, Fail)]
pub enum FlokiError {
    #[fail(display = "No floki.yaml found in tree")]
    ProblemFindingConfigYaml {},

    #[fail(display = "Could not normalize the file path '{}': {}", name, error)]
    ProblemNormalizingFilePath { name: String, error: io::Error },

    #[fail(
        display = "There was a problem opening the configuration file '{}': {}",
        name, error
    )]
    ProblemOpeningConfigYaml { name: String, error: io::Error },

    #[fail(
        display = "There was a problem parsing the configuration file '{}': {}",
        name, error
    )]
    ProblemParsingConfigYaml {
        name: String,
        error: serde_yaml::Error,
    },

    #[fail(display = "Running docker command failed with error: {}", error)]
    FailedToLaunchDocker { error: io::Error },

    #[fail(display = "Failed to complete docker command with error: {}", error)]
    FailedToCompleteDockerCommand { error: io::Error },

    #[fail(display = "Failed to pull docker image '{}': {}", image, exit_status)]
    FailedToPullImage {
        image: String,
        exit_status: FlokiSubprocessExitStatus,
    },

    #[fail(display = "Failed to build docker image '{}': {}", image, exit_status)]
    FailedToBuildImage {
        image: String,
        exit_status: FlokiSubprocessExitStatus,
    },

    #[fail(display = "Failed to check existence of image '{}': {}", image, error)]
    FailedToCheckForImage { image: String, error: io::Error },

    #[fail(display = "Failed to find the key '{}' in file '{}'", key, file)]
    FailedToFindYamlKey { key: String, file: String },

    #[fail(display = "Running container failed: {}", exit_status)]
    RunContainerFailed {
        exit_status: FlokiSubprocessExitStatus,
    },

    #[fail(display = "Unable to forward ssh socket - cannot find SSH_AUTH_SOCK in environment")]
    NoSshAuthSock {},
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
#[derive(Debug, Fail)]
pub enum FlokiInternalError {
    #[fail(
        display = "An internal assertion failed '{}'.  This is probably a bug!",
        description
    )]
    InternalAssertionFailed { description: String },
}

/// Errors made by floki users.
#[derive(Debug, Fail)]
pub enum FlokiUserError {
    #[fail(
        display = "Invalid verbosity setting of {}. Use a setting between 0 and 3 (-vvv)",
        setting
    )]
    InvalidVerbositySetting { setting: u8 },
}
