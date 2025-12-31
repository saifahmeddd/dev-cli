use crate::modules::snip::storage::{SnippetStore, Snippet};
use crate::utils::process;
use anyhow::{Result, Context};
use dialoguer::{Select, Input};
use console::style;
use regex::Regex;

pub fn run_snippet(name: Option<String>, args: Vec<String>) -> Result<()> {
    let mut store = SnippetStore::load()?;
    let snippets = store.list();

    if snippets.is_empty() {
        println!("No snippets found. Add one with `dev snip add`.");
        return Ok(());
    }

    let snippet_name = match name {
        Some(n) => n,
        None => {
            // Interactive selection
            let names: Vec<String> = snippets.iter().map(|s| s.name.clone()).collect();
            let selection = Select::new()
                .with_prompt("Select a snippet to run")
                .items(&names)
                .default(0)
                .interact()?;
            names[selection].clone()
        }
    };

    let snippet = store.get_mut(&snippet_name).ok_or_else(|| anyhow::anyhow!("Snippet '{}' not found", snippet_name))?;
    
    // Increment usage
    snippet.usage_count += 1;
    let content = snippet.content.clone();
    
    // Save usage inc
    store.save()?;

    // Variable substitution
    // Simple approach: replace {{VAR}} or $VAR with values from args or prompt
    // For now, let's just append args? Or use regex substitution.
    // Spec says: dev snip run deploy --env=prod
    // This implies we pass arguments to the command or substitute.
    
    // Implementation: simple shell execution.
    // If snippet is "echo $ENV", and we run it, shell handles $ENV if exported.
    // If we want template substitution like " deploy --target {{TARGET}}", we need parsing.
    
    // Let's support {{VAR}} substitution by prompting if not provided in args (args support matches spec better but parsing --key=val is extra work).
    // For MVP, just run the command string and append args?
    // "deploy" might be "ansible-playbook deploy.yml". "dev snip run deploy --foo" -> "ansible-playbook deploy.yml --foo".
    // That seems most standard for "snippets" unless it's a template system.
    
    let mut final_command = content;
    if !args.is_empty() {
        final_command.push_str(" ");
        final_command.push_str(&args.join(" "));
    }

    println!("{}", style(format!("Running: {}", final_command)).cyan());
    
    // Execute
    // Use sh -c to allow multiple words/pipes
    let shell = crate::utils::platform::get_shell();
    let status = std::process::Command::new(shell)
        .arg("-c")
        .arg(&final_command)
        .status()
        .context("Failed to execute snippet")?;

    if !status.success() {
        println!("{}", style("Snippet execution failed.").red());
    }

    Ok(())
}
