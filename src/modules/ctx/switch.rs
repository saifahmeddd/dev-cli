use crate::modules::ctx::storage::ContextStore;
use crate::utils::{platform};
use anyhow::{Result, Context as AnyhowContext};
use std::process::Stdio;
use console::style;

pub fn switch_context(name: &str) -> Result<()> {
    let store = ContextStore::load()?;
    let context = store.get(name).ok_or_else(|| anyhow::anyhow!("Context '{}' not found", name))?;

    println!("{}", style(format!("Layering into context '{}'...", name)).bold().cyan());
    if let Some(desc) = &context.description {
        println!("📝 {}", style(desc).italic());
    }
    if let Some(ptype) = &context.project_type {
        println!("🚀 Project Type: {}", style(ptype).yellow());
    }
    println!("📂 Path: {:?}", context.path);
    if let Some(branch) = &context.git_branch {
        println!("🌿 Git Branch: {}", style(branch).green());
    }

    // Platform specific shell
    let shell = platform::get_shell();
    let mut command = std::process::Command::new(&shell);
    
    // Windows specific flags if needed?
    // Usually cmd.exe /K or powershell -NoExit, but we want to replace/spawn a new interactive one.
    // Standard execution of shell usually enters interactive mode.
    
    command.current_dir(&context.path);
    command.envs(&context.env_vars);
    command.env("DEV_CLI_CONTEXT", &context.name);
    
    command.stdin(Stdio::inherit())
           .stdout(Stdio::inherit())
           .stderr(Stdio::inherit());

    let mut child = command.spawn().context("Failed to spawn shell")?;
    
    child.wait()?;
    
    println!("Exited context '{}'.", name);

    Ok(())
}
