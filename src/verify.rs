use crate::cli::Cli;
use crate::config::FlokiConfig;
use crate::errors;
use quicli::prelude::*;


pub(crate) fn verify_command(args: &Cli, config: &FlokiConfig) -> Result<()> {
    if config.docker_switches.len() > 0 && !args.local {
        Err(errors::FlokiError::NonLocalDockerSwitches{})?
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::image::Image::Name;
    use crate::config::Shell::Shell;

    #[test]
    fn test_nonlocal_docker_switches_non_empty() {
        let args = Cli {
            config_file: "floki.yaml".into(),
            local: false,
            // Dummy verbosity which we don't use. This is gross.
            verbosity: unsafe { std::mem::transmute(0 as u8) },
            subcommand: None
        };

        let config = FlokiConfig {
            image: Name("foo".into()),
            init: vec![],
            shell: Shell("bash".into()),
            mount: "/mnt".into(),
            docker_switches: vec!["dummy".into(), "switches".into()],
            forward_ssh_agent: false,
            dind: false,
            forward_user: false
        };

        let res = verify_command(&args, &config);
        assert!(res.is_err());
    }

    #[test]
    fn test_local_docker_switches_non_empty() {
        let args = Cli {
            config_file: "floki.yaml".into(),
            local: true,
            // Dummy verbosity which we don't use. This is gross.
            verbosity: unsafe { std::mem::transmute(0 as u8) },
            subcommand: None
        };

        let config = FlokiConfig {
            image: Name("foo".into()),
            init: vec![],
            shell: Shell("bash".into()),
            mount: "/mnt".into(),
            docker_switches: vec!["dummy".into(), "switches".into()],
            forward_ssh_agent: false,
            dind: false,
            forward_user: false
        };

        let res = verify_command(&args, &config);
        assert!(res.is_ok());
    }
}
