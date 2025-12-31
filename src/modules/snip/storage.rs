use serde_derive::{Serialize, Deserialize};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snippet {
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub usage_count: u32,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SnippetStore {
    pub snippets: HashMap<String, Snippet>,
}

impl SnippetStore {
    pub fn load() -> Result<Self> {
        let path = get_storage_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let store: SnippetStore = serde_json::from_str(&content)?;
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

    pub fn add(&mut self, snippet: Snippet) {
        self.snippets.insert(snippet.name.clone(), snippet);
    }
    
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Snippet> {
        self.snippets.get_mut(name)
    }

    pub fn list(&self) -> Vec<&Snippet> {
        self.snippets.values().collect()
    }
}

fn get_storage_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".local").join("share").join("dev-cli").join("snippets.json"))
}
