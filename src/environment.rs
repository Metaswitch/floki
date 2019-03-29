/// Query the current user environment
use quicli::prelude::*;
use std::process::Command;
use std::env;


pub struct Environment {
    pub user_details: (String, String),
    pub current_directory: String,
    pub ssh_agent_socket: Option<String>
}


impl Environment {
    pub fn gather() -> Result<Self> {
        Ok(Environment{
            user_details: get_user_details()?,
            current_directory: get_current_working_directory()?,
            ssh_agent_socket: get_ssh_agent_socket_path()
        })
    }
}


fn run_and_get_raw_output(cmd: &mut Command) -> Result<String> {
    let output = String::from_utf8(cmd.output()?.stdout)?;
    Ok(output.trim_end().into())
}

pub fn get_user_details() -> Result<(String, String)> {
    let user = run_and_get_raw_output(Command::new("id").arg("-u"))?;
    debug!("User's current id: {:?}", user);
    let group = run_and_get_raw_output(Command::new("id").arg("-g"))?;
    debug!("User's current group: {:?}", group);
    Ok((user, group))
}

/// Get the current working directory as a String
fn get_current_working_directory() -> Result<String> {
    Ok(format!("{}", env::current_dir()?.display()))
}


/// Get the path of the ssh agent socket from the SSH_AUTH_SOCK
/// environment variable
fn get_ssh_agent_socket_path() -> Option<String> {
    match env::var("SSH_AUTH_SOCK") {
        Ok(path) => Some(path),
        Err(_) => None
    }
}
