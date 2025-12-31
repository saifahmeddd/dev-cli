use crate::cli::{SnipArgs, SnipCommands};
use anyhow::Result;

pub mod storage;
pub mod add;
pub mod run;
pub mod search;

pub async fn handle_snip(args: &SnipArgs) -> Result<()> {
    match &args.command {
        Some(SnipCommands::Add) => {
             add::add_snippet()?;
        }
        Some(SnipCommands::Run { name, args }) => {
             run::run_snippet(name.clone(), args.clone())?;
        }
        Some(SnipCommands::List) => {
             search::search_snippets()?;
        }
        Some(SnipCommands::Search) => {
             // Re-use list/search logic
             search::search_snippets()?;
        }
        None => {
             search::search_snippets()?;
        }
    }
    Ok(())
}
