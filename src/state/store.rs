use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project type detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Java,
    Unknown,
}

/// Context data structure - represents a complete project context
#[derive(Debug, Clone)]
pub struct Context {
    pub name: String,
    pub working_dir: String,
    pub git_branch: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub on_switch: Option<String>,
    // Project-oriented fields
    pub project_type: Option<ProjectType>,
    pub dependencies: HashMap<String, String>, // package name -> version/file path
    pub virtual_env: Option<String>, // path to virtual environment
    pub important_files: Vec<String>, // relative paths to important files
    pub important_dirs: Vec<String>, // relative paths to important directories
    pub package_manager: Option<String>, // npm, cargo, pip, go, etc.
}

/// WTF explanation data
#[derive(Debug, Clone)]
pub struct WtfExplanation {
    pub fingerprint: String,
    pub cause: String,
    pub suggested_fix: String,
    pub confidence: f64,
}

/// Snippet data structure
#[derive(Debug, Clone)]
pub struct Snippet {
    pub name: String,
    pub command: String,
    pub tags: Vec<String>,
}

/// Trait for state storage operations
/// Represents learned behavior and persistent state
/// Note: Only requires Send (not Sync) since this is a single-threaded CLI application
pub trait StateStore: Send {
    /// Initialize the state store
    fn init(&mut self) -> Result<()>;
    
    /// Close the state store
    fn close(&mut self) -> Result<()>;
    
    // Context operations
    fn save_context(&mut self, context: Context) -> Result<()>;
    fn get_context(&self, name: &str) -> Result<Option<Context>>;
    fn list_contexts(&self) -> Result<Vec<String>>;
    fn delete_context(&mut self, name: &str) -> Result<()>;
    
    // WTF operations
    fn get_wtf_explanation(&self, fingerprint: &str) -> Result<Option<WtfExplanation>>;
    fn save_wtf_explanation(&mut self, explanation: WtfExplanation) -> Result<()>;
    
    // Snippet operations
    fn save_snippet(&mut self, snippet: Snippet) -> Result<()>;
    fn get_snippet(&self, name: &str) -> Result<Option<Snippet>>;
    fn list_snippets(&self, tag_filter: Option<&str>) -> Result<Vec<Snippet>>;
    fn delete_snippet(&mut self, name: &str) -> Result<()>;
}

