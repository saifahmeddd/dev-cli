use crate::app::AppContext;
use crate::cli::{GitArgs, GitCommand};
use crate::utils::llm::call_llm_json;
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Confirm};
use std::process::Command;

pub fn run(args: GitArgs, _app_ctx: &mut AppContext) -> Result<()> {
    match args.command {
        GitCommand::SmartCommit { hint, dry_run } => smart_commit(hint, dry_run),
        GitCommand::Review => review(),
        GitCommand::Cleanup { dry_run } => cleanup(dry_run),
        GitCommand::Stats => stats(),
        GitCommand::Worktree { name } => worktree(name),
    }
}

fn smart_commit(hint: Option<String>, dry_run: bool) -> Result<()> {
    // 1. Get staged changes
    let mut diff_output = Command::new("git")
        .args(&["diff", "--staged"])
        .output()
        .context("Failed to run git diff --staged")?;
        
    // If no staged changes, try unstaged to be helpful (but warn)
    if diff_output.stdout.is_empty() {
        println!("No staged changes found. using unstaged changes for context...");
        diff_output = Command::new("git")
            .args(&["diff"])
            .output()
            .context("Failed to run git diff")?;
            
        if diff_output.stdout.is_empty() {
            return Err(anyhow::anyhow!("No changes to commit"));
        }
    }
    
    let diff = String::from_utf8_lossy(&diff_output.stdout);
    if diff.len() > 10000 {
        println!("⚠️  Diff is too large ({} chars), truncating...", diff.len());
    }
    let truncated_diff = diff.chars().take(10000).collect::<String>();
    
    // 2. Call LLM
    let hint_text = hint.map(|h| format!("User hint: '{}'", h)).unwrap_or_default();
    let prompt = format!(
        "Generate a conventional commit message for these changes. {} \n\nOutput JSON with 'commit_message' field.\n\nDiff:\n{}", 
        hint_text, truncated_diff
    );
    
    println!("🤖 Generating commit message...");
    let response = call_llm_json(
        "You are an expert developer. Generate valid conventional commits. Return JSON.", 
        &prompt
    ).context("Failed to generate commit message")?;
    
    let message = response["commit_message"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid response"))?
        .to_string();
        
    println!("\nSuggested Commit Message:\n{}", message);
    
    if dry_run {
        return Ok(());
    }
    
    // 3. Confirm and Commit
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to commit with this message?")
        .default(true)
        .interact()? 
    {
        // If we were looking at unstaged, stage them now? No, that's dangerous.
        // But if `git diff --staged` was empty, we can't commit.
        // Let's re-verify staged.
        let check = Command::new("git").args(&["diff", "--staged", "--quiet"]).status()?;
        if check.success() { 
             if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("No staged changes. Stage all changes and commit?")
                .default(false)
                .interact()? 
            {
                Command::new("git").arg("add").arg(".").status()?;
            } else {
                return Ok(());
            }
        }
        
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(&message)
            .status()
            .context("Failed to commit")?;
            
        if status.success() {
            println!("✅ Committed!");
        } else {
            eprintln!("❌ Commit failed");
        }
    }
    
    Ok(())
}

fn review() -> Result<()> {
    let items = vec![
        "Code implementation matches requirements",
        "Tests passed (cargo test)",
        "Linter passed (cargo clippy)",
        "Code is formatted (cargo fmt)",
        "No hardcoded secrets",
        "Error handling is robust",
        "Documentation updated",
    ];
    
    println!("\n📝 Interactive Review Checklist\n");
    
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&items)
        .interact()?;
        
    if selections.len() == items.len() {
        println!("\n✨ Ready to merge! Good job.");
    } else {
        println!("\n⚠️  You have {} pending items.", items.len() - selections.len());
    }
    
    Ok(())
}

fn cleanup(dry_run: bool) -> Result<()> {
    // 1. Prune remotes
    println!("🧹 Pruning stale remote branches...");
    if !dry_run {
        Command::new("git").args(&["remote", "prune", "origin"]).status()?;
    }
    
    // 2. Delete merged branches
    println!("🧹 Checking for merged branches...");
    let output = Command::new("git")
        .args(&["branch", "--merged"])
        .output()?;
        
    let branches = String::from_utf8_lossy(&output.stdout);
    let mut to_delete = Vec::new();
    
    for branch in branches.lines() {
        let b = branch.trim();
        if b.starts_with("*") { continue; } // Current branch
        if b == "main" || b == "master" || b == "dev" || b == "develop" { continue; }
        to_delete.push(b.to_string());
    }
    
    if to_delete.is_empty() {
        println!("No merged branches to clean up.");
    } else {
        println!("Found {} merged branches: {:?}", to_delete.len(), to_delete);
        
        if dry_run {
            println!("Dry run: would delete branches.");
        } else {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Delete these branches?")
                .default(false)
                .interact()?
            {
                for branch in to_delete {
                    Command::new("git").args(&["branch", "-d", &branch]).status()?;
                    println!("Deleted {}", branch);
                }
            }
        }
    }
    
    Ok(())
}

fn stats() -> Result<()> {
    println!("📊 Contribution Stats:\n");
    Command::new("git")
        .args(&["shortlog", "-sn", "--all", "--no-merges"])
        .spawn()?
        .wait()?;
    Ok(())
}

fn worktree(name: String) -> Result<()> {
    println!("🌲 Creating worktree '{}'...", name);
    
    let path = format!("../{}", name);
    
    let status = Command::new("git")
        .args(&["worktree", "add", &path, &name])
        .status()?;
        
    if status.success() {
        println!("✅ Worktree created at {}", path);
        // Optional: Copy .env
        if std::path::Path::new(".env").exists() {
             std::fs::copy(".env", format!("{}/.env", path)).ok();
             println!("dependencies copied (.env)");
        }
    } else {
        eprintln!("❌ Failed to create worktree");
    }
    
    Ok(())
}
