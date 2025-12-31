use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::state::StateStore;
use anyhow::Result;

/// Application context containing config and state
pub struct AppContext {
    pub config: Config,
    pub state: Box<dyn StateStore>,
}

impl AppContext {
    pub fn new(config: Config, state: Box<dyn StateStore>) -> Self {
        AppContext { config, state }
    }
}

/// Central command dispatcher
/// Routes commands to appropriate module handlers
pub fn run(cli: Cli, mut ctx: AppContext) -> Result<()> {
    match cli.command {
        Commands::Ctx(args) => {
            crate::modules::ctx::run(args, &mut ctx)
        }
        Commands::Wtf(args) => {
            crate::modules::wtf::run(args, &mut ctx)
        }
        Commands::Snip(args) => {
            crate::modules::snip::run(args, &mut ctx)
        }
        Commands::Config(_args) => {
            println!("config: Configuration management (placeholder - not yet implemented)");
            Ok(())
        }
    }
}

