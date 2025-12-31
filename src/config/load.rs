use crate::config::model::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Load configuration from user's home directory or fallback to bundled default
pub fn load_config() -> Result<Config> {
    let user_config_path = get_user_config_path()?;
    
    // Try to load user config first
    if user_config_path.exists() {
        let content = fs::read_to_string(&user_config_path)
            .with_context(|| format!("Failed to read config from {:?}", user_config_path))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {:?}", user_config_path))?;
        return Ok(config);
    }
    
    // Fallback to bundled default
    load_default_config()
}

/// Get the path to the user's config file
fn get_user_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .context("Could not determine home directory")?;
    Ok(home.join(".dev").join("config.toml"))
}

/// Load the bundled default configuration
fn load_default_config() -> Result<Config> {
    // In a real implementation, this would be embedded at compile time
    // For now, we'll try to read from config/default.toml relative to the binary
    // This is a placeholder - in production, consider using include_str! macro
    let default_content = include_str!("../../config/default.toml");
    let config: Config = toml::from_str(default_content)
        .context("Failed to parse default config")?;
    Ok(config)
}

