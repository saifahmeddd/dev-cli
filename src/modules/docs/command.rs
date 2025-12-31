use crate::app::AppContext;
use crate::cli::{DocsArgs, DocsCommand};
use crate::utils::llm::call_llm_json;
use anyhow::{Context, Result};
use std::fs;

pub fn run(args: DocsArgs, _app_ctx: &mut AppContext) -> Result<()> {
    match args.command {
        DocsCommand::Generate { file } => generate_docs(&file),
        DocsCommand::Readme => generate_readme(),
        DocsCommand::Api => {
            println!("API documentation generation not implemented yet.");
            Ok(())
        }
        DocsCommand::Sync => {
            println!("Docs sync not implemented yet.");
            Ok(())
        }
        DocsCommand::Search { query } => {
             println!("Searching docs for '{}'...", query);
             Ok(())
        }
    }
}

fn generate_docs(file_path: &str) -> Result<()> {
    println!("📖 Generating documentation for {}...", file_path);
    
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file {}", file_path))?;
        
    let prompt = format!(
        "Generate detailed documentation for the following code. Include overview, function descriptions, and usage examples.\n\nCode:\n{}",
        content
    );
    
    let _response = call_llm_json(
        "You are a technical writer. Generate markdown documentation. Return JSON with 'markdown' field.", 
        &prompt
    )?;
    
    // Actually call_llm_json returns a Value.
    let markdown = _response["markdown"].as_str().unwrap_or("Failed to generate documentation.").to_string();
    
    println!("\nGenerated Documentation:\n");
    println!("{}", markdown);
    
    Ok(())
}

fn generate_readme() -> Result<()> {
    println!("📖 Generating README.md...");
    
    // Simple tree structure (limited depth)
    // In a real app, we'd traverse directories
    let entries = fs::read_dir(".")?;
    let mut tree_str = String::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
             tree_str.push_str(&format!("- {}\n", name.to_string_lossy()));
        }
    }
    
    let prompt = format!(
        "Generate a README.md for a project with the following file structure. guess the project type and purpose.\n\nStructure:\n{}",
        tree_str
    );
    
     let _response = call_llm_json(
        "You are a technical writer. Generate README.md. Return JSON with 'markdown' field.", 
        &prompt
    )?;
    
    let markdown = _response["markdown"].as_str().unwrap_or("Failed to generate README.").to_string();
    
    println!("\nGenerated README:\n");
    println!("{}", markdown);
    
    Ok(())
}
