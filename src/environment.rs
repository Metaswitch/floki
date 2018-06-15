/// Query the current user environment
use quicli::prelude::*;
use std::process::Command;

fn run_and_get_raw_output(cmd: &mut Command) -> Result<String> {
    let output = String::from_utf8(cmd.output()?.stdout)?;
    Ok(output.trim_right().into())
}

pub fn get_user_details() -> Result<(String, String)> {
    let user = run_and_get_raw_output(Command::new("id").arg("-u"))?;
    debug!("User's current id: {:?}", user);
    let group = run_and_get_raw_output(Command::new("id").arg("-g"))?;
    debug!("User's current group: {:?}", group);
    Ok((user.into(), group.into()))
}
