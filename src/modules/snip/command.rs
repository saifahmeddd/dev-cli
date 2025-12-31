use crate::app::AppContext;
use crate::cli::{SnipArgs, SnipCommand};
use crate::state::store::Snippet;
use anyhow::{Context as AnyhowContext, Result};
use regex;
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::Command;

/// Run the snippet management command
pub fn run(args: SnipArgs, app_ctx: &mut AppContext) -> Result<()> {
    match args.command {
        SnipCommand::Add { name, command, tags } => add_snippet(name, command, tags, app_ctx),
        SnipCommand::List { tag } => list_snippets(tag, app_ctx),
        SnipCommand::Run { name, vars, dry_run } => run_snippet(name, vars, dry_run, app_ctx),
    }
}

fn add_snippet(
    name: String,
    command: String,
    tags: Option<String>,
    app_ctx: &mut AppContext,
) -> Result<()> {
    let tag_list = if let Some(tags_str) = tags {
        tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };
    
    let snippet = Snippet {
        name: name.clone(),
        command,
        tags: tag_list,
    };
    
    app_ctx.state.save_snippet(snippet)
        .with_context(|| format!("Failed to save snippet '{}'", name))?;
    
    println!("Saved snippet '{}'", name);
    Ok(())
}

fn list_snippets(tag_filter: Option<String>, app_ctx: &mut AppContext) -> Result<()> {
    let snippets = app_ctx.state.list_snippets(tag_filter.as_deref())
        .context("Failed to list snippets")?;
    
    if snippets.is_empty() {
        println!("No snippets found.");
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        
        for snippet in snippets {
            write!(handle, "{}", snippet.name)?;
            if !snippet.tags.is_empty() {
                write!(handle, " [{}]", snippet.tags.join(", "))?;
            }
            writeln!(handle)?;
            writeln!(handle, "  {}", snippet.command)?;
        }
    }
    
    Ok(())
}

fn run_snippet(
    name: String,
    vars: Vec<String>,
    dry_run: bool,
    app_ctx: &mut AppContext,
) -> Result<()> {
    let snippet = app_ctx.state.get_snippet(&name)
        .context("Failed to get snippet")?
        .ok_or_else(|| anyhow::anyhow!("Snippet '{}' not found", name))?;
    
    // Parse variables
    let var_map = parse_vars(vars)?;
    
    // Substitute variables in command
    let command = substitute_variables(&snippet.command, &var_map)?;
    
    if dry_run {
        println!("Would run: {}", command);
        return Ok(());
    }
    
    // Execute command via system shell
    let shell = crate::platform::detect_shell().unwrap_or(crate::platform::Shell::Bash);
    
    let status = match shell {
        crate::platform::Shell::Bash | crate::platform::Shell::Zsh => {
            Command::new("sh")
                .arg("-c")
                .arg(&command)
                .status()
        }
        crate::platform::Shell::Fish => {
            Command::new("fish")
                .arg("-c")
                .arg(&command)
                .status()
        }
        crate::platform::Shell::PowerShell => {
            Command::new("powershell")
                .arg("-Command")
                .arg(&command)
                .status()
        }
        _ => {
            Command::new("sh")
                .arg("-c")
                .arg(&command)
                .status()
        }
    }?;
    
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    
    Ok(())
}

fn parse_vars(vars: Vec<String>) -> Result<HashMap<String, String>> {
    let mut var_map = HashMap::new();
    
    for var_str in vars {
        let parts: Vec<&str> = var_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid variable format: {}. Expected key=value", var_str));
        }
        var_map.insert(parts[0].to_string(), parts[1].to_string());
    }
    
    Ok(var_map)
}

fn substitute_variables(command: &str, vars: &HashMap<String, String>) -> Result<String> {
    let mut result = command.to_string();
    
    // Find all {var} patterns
    let re = regex::Regex::new(r"\{([^}]+)\}")?;
    
    for cap in re.captures_iter(command) {
        let var_name = cap.get(1).unwrap().as_str();
        let replacement = vars.get(var_name)
            .ok_or_else(|| anyhow::anyhow!("Variable '{}' not provided", var_name))?;
        
        result = result.replace(&format!("{{{}}}", var_name), replacement);
    }
    
    Ok(result)
}
