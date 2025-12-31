mod cli;
mod config;
mod modules;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    // Load config (ignoring errors for now or handling them)
    let _config = config::load_config().unwrap_or_default();


    match &cli.command {
        Some(Commands::Ctx(args)) => modules::ctx::handle_ctx(args).await?,
        Some(Commands::Wtf(args)) => modules::wtf::handle_wtf(args).await?,

        Some(Commands::Snip(args)) => modules::snip::handle_snip(args).await?,
        Some(Commands::Config(args)) => config::handle_config(args).await?,
        None => {
             // Default behavior if no command? maybe help
        }
    }

    Ok(())
}
