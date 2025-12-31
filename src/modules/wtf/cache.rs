use serde_derive::{Serialize, Deserialize};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct ErrorCache {
    pub errors: HashMap<String, String>,
}

impl ErrorCache {
    pub fn load() -> Result<Self> {
        let path = get_cache_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let cache: ErrorCache = serde_json::from_str(&content)?;
        Ok(cache)
    }

    pub fn save(&self) -> Result<()> {
        let path = get_cache_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get(&self, error_msg: &str) -> Option<String> {
        // Simple exact matching for now. 
        // In real world, might need fuzzy match or stripping timestamps.
        self.errors.get(error_msg).cloned()
    }

    pub fn insert(&mut self, error_msg: String, explanation: String) {
        self.errors.insert(error_msg, explanation);
    }
}

fn get_cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".cache").join("dev-cli").join("wtf_cache.json"))
}
