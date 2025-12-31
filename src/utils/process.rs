use anyhow::{Result, Context};
use std::process::{Command, Stdio, Child};

pub fn run_command(cmd: &str, args: &[&str], cwd: Option<&std::path::Path>) -> Result<String> {
    let mut command = Command::new(cmd);
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    
    let output = command.output().with_context(|| format!("Failed to execute command: {}", cmd))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}\nStderr: {}", output.status, stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn spawn_command(cmd: &str, args: &[&str], cwd: Option<&std::path::Path>) -> Result<Child> {
    let mut command = Command::new(cmd);
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    
    // Inherit stdio for interactive commands or long running processes
    command.stdout(Stdio::inherit())
           .stderr(Stdio::inherit())
           .stdin(Stdio::inherit());

    let child = command.spawn().with_context(|| format!("Failed to spawn command: {}", cmd))?;
    Ok(child)
}
