use crate::cli::{CtxArgs, CtxCommands};
use anyhow::Result;

pub mod save;
pub mod switch;
pub mod storage;

pub async fn handle_ctx(args: &CtxArgs) -> Result<()> {
    match &args.command {
        Some(CtxCommands::Switch { name }) => {
            switch::switch_context(name)?;
        }
        Some(CtxCommands::Save { name }) => {
            save::save_context(name.clone())?;
        }
        Some(CtxCommands::Show { name }) => {
            let store = storage::ContextStore::load()?;
            if let Some(context) = store.get(name) {
                println!("{}", console::style(format!("Context: {}", context.name)).bold().cyan());
                println!("  Path: {:?}", context.path);
                println!("  Type: {}", context.project_type.as_deref().unwrap_or("Unknown"));
                if let Some(desc) = &context.description {
                    println!("  Desc: {}", desc);
                }
                if let Some(branch) = &context.git_branch {
                    println!("  Git:  {}", branch);
                }
                println!("  Env Vars: {} saved", context.env_vars.len());
                // Optionally list them or a few of them?
                // For now just count is fine, or verbose mode might show all.
                // Let's show up to 5 keys as preview.
                for (i, (k, v)) in context.env_vars.iter().take(100).enumerate() {
                    println!("    {}: {}", k, v);
                }
                if context.env_vars.len() > 100 {
                    println!("    ... and {} more", context.env_vars.len() - 100);
                }
            } else {
                println!("Context '{}' not found.", name);
            }
        }
        Some(CtxCommands::List) => {
            let store = storage::ContextStore::load()?;
            if store.contexts.is_empty() {
                println!("No saved contexts.");
            } else {
                println!("Saved contexts:");
                for context in store.list() {
                    println!("  - {}", context.name);
                }
            }
        }
        Some(CtxCommands::Current) => {
             match std::env::var("DEV_CLI_CONTEXT") {
                Ok(name) => println!("Current context: {}", console::style(name).green().bold()),
                Err(_) => println!("No active dev-cli context."),
            }
        }
        Some(CtxCommands::Delete { name }) => {
            let mut store = storage::ContextStore::load()?;
            if store.remove(name).is_some() {
                store.save()?;
                println!("Deleted context '{}'.", name);
            } else {
                println!("Context '{}' not found.", name);
            }
        }
        None => {
            let store = storage::ContextStore::load()?;
            if store.contexts.is_empty() {
                println!("No saved contexts.");
            } else {
                println!("Saved contexts:");
                for context in store.list() {
                     let ptype = context.project_type.as_deref().unwrap_or("generic");
                     println!("  - {} ({})", context.name, console::style(ptype).dim());
                }
            }
        }
    }
    Ok(())
}

