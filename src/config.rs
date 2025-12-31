use serde_derive::{Deserialize, Serialize};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub theme: String,
    pub editor: String,
}

pub async fn handle_config(args: &crate::cli::ConfigArgs) -> Result<()> {
    println!("Handling config command: {:?}", args);
    Ok(())
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    let content = toml::to_string(config)?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(config_path, content)?;
    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".config").join("dev-cli").join("config.toml"))
}
