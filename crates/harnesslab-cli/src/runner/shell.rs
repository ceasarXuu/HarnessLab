use anyhow::{Result, bail};
use std::path::Path;
use std::process::Command;

pub(super) fn run_shell(cwd: &Path, command: &str) -> Result<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .status()?;
    if !status.success() {
        bail!("command failed: {command}");
    }
    Ok(())
}
