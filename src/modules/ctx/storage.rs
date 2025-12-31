use serde_derive::{Serialize, Deserialize};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    pub name: String,
    pub path: PathBuf,
    pub env_vars: HashMap<String, String>,
    pub git_branch: Option<String>,
    pub project_type: Option<String>,
    pub description: Option<String>,
    // Future: editor state, open files, etc.
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ContextStore {
    pub contexts: HashMap<String, Context>,
}

impl ContextStore {
    pub fn load() -> Result<Self> {
        let path = get_storage_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let store: ContextStore = serde_json::from_str(&content)?;
        Ok(store)
    }

    pub fn save(&self) -> Result<()> {
        let path = get_storage_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add(&mut self, context: Context) {
        self.contexts.insert(context.name.clone(), context);
    }

    pub fn get(&self, name: &str) -> Option<&Context> {
        self.contexts.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Context> {
        self.contexts.remove(name)
    }
    
    pub fn list(&self) -> Vec<&Context> {
        self.contexts.values().collect()
    }
}

fn get_storage_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".local").join("share").join("dev-cli").join("contexts.json"))
}
