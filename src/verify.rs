use crate::config::FlokiConfig;
use crate::errors;
use failure::Error;

pub(crate) fn verify_command(local: bool, config: &FlokiConfig) -> Result<(), Error> {
    if config.docker_switches.len() > 0 && !local {
        Err(errors::FlokiError::NonLocalDockerSwitches {})?
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::DindConfig;
    use crate::config::Shell::Shell;
    use crate::image::Image::Name;

    use std::collections::BTreeMap;

    fn get_test_config(docker_switches: Vec<String>) -> FlokiConfig {
        FlokiConfig {
            image: Name("foo".into()),
            init: vec![],
            shell: Shell("bash".into()),
            mount: "/mnt".into(),
            docker_switches,
            forward_ssh_agent: false,
            dind: DindConfig::deactivated(),
            forward_user: false,
            volumes: BTreeMap::new(),
        }
    }

    #[test]
    fn test_nonlocal_docker_switches_non_empty() {
        let config = get_test_config(vec!["dummy".into(), "switches".into()]);
        let res = verify_command(false, &config);
        assert!(res.is_err());
    }

    #[test]
    fn test_local_docker_switches_non_empty() {
        let config = get_test_config(vec!["dummy".into(), "switches".into()]);
        let res = verify_command(true, &config);
        assert!(res.is_ok());
    }
}
