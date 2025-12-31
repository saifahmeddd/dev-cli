use anyhow::{Result, anyhow};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn capture_last_error() -> Result<String> {
    // Attempt to read from a log file if configured
    if let Ok(log_path) = env::var("DEV_CLI_LAST_ERROR_LOG") {
        let path = PathBuf::from(log_path);
        if path.exists() {
            let content = fs::read_to_string(path)?;
            if !content.trim().is_empty() {
                return Ok(content);
            }
        }
    }

    // Fallback: Check if we are in a dev-cli wrapper context/shell?
    // Unlikely for now.
    
    Err(anyhow!("Could not automatically capture last error. Please provide the error message explicitly: `dev wtf \"error message\"` or configure shell integration."))
}
