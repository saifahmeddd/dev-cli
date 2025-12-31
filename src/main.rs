mod app;
mod cli;
mod config;
mod modules;
mod platform;
mod state;

use anyhow::{Context, Result};
use clap::Parser;
use env_logger;
use state::store::StateStore;

fn main() {
    // Initialize logging from RUST_LOG environment variable
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Load configuration
    let config = config::load_config()
        .context("Failed to load configuration")?;
    
    // Initialize state store
    let mut state_store = state::SqliteStore::new()
        .context("Failed to initialize state store")?;
    state_store.init()
        .context("Failed to initialize state store")?;
    
    // Parse CLI arguments
    let cli = cli::Cli::parse();
    
    // Create application context
    let app_ctx = app::AppContext::new(config, Box::new(state_store));
    
    // Dispatch to app router
    app::run(cli, app_ctx)
        .context("Command execution failed")
}

