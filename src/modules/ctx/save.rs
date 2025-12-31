use crate::modules::ctx::storage::{Context, ContextStore};
use crate::utils::{env, process};
use anyhow::Result;
use std::env as std_env;
use std::path::Path;

pub fn capture_context(name: String) -> Result<Context> {
    let path = std_env::current_dir()?;
    let env_vars = env::get_current_env();
    
    let git_branch = match process::run_command("git", &["rev-parse", "--abbrev-ref", "HEAD"], Some(&path)) {
        Ok(output) => Some(output.trim().to_string()),
        Err(_) => None, 
    };

    let project_type = detect_project_type(&path);
    let description = Some(format!("Auto-saved context for {} project '{}'", 
        project_type.as_deref().unwrap_or("generic"), 
        name));

    Ok(Context {
        name,
        path,
        env_vars,
        git_branch,
        project_type,
        description,
    })
}

pub fn save_context(name: String) -> Result<()> {
    let mut store = ContextStore::load()?;
    let context = capture_context(name.clone())?;
    
    store.add(context);
    store.save()?;
    
    println!("Context '{}' saved.", name);
    Ok(())
}

fn detect_project_type(path: &Path) -> Option<String> {
    if path.join("Cargo.toml").exists() {
        return Some("Rust".to_string());
    }
    if path.join("package.json").exists() {
        return Some("Node.js".to_string());
    }
    if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
        return Some("Python".to_string());
    }
    if path.join("go.mod").exists() {
        return Some("Go".to_string());
    }
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        return Some("Java".to_string());
    }
    None
}
