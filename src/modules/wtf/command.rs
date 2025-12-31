use crate::app::AppContext;
use crate::cli::WtfArgs;
use crate::state::store::WtfExplanation;
use anyhow::{Context as AnyhowContext, Result};
use regex;
use sha2::{Digest, Sha256};
use std::io::{self, Write};

/// Run the error explanation (wtf) command
pub fn run(args: WtfArgs, app_ctx: &mut AppContext) -> Result<()> {
    let error_text = args.error_text.trim();
    
    if error_text.is_empty() {
        return Err(anyhow::anyhow!("Error text cannot be empty"));
    }
    
    // Generate fingerprint
    let fingerprint = generate_fingerprint(error_text);
    
    // Check cache
    if let Some(explanation) = app_ctx.state.get_wtf_explanation(&fingerprint)
        .context("Failed to check WTF cache")? {
        print_explanation(&explanation);
        return Ok(());
    }
    
    // Generate deterministic explanation stub
    let explanation = generate_explanation(error_text, &fingerprint);
    
    // Save to cache
    app_ctx.state.save_wtf_explanation(explanation.clone())
        .context("Failed to save WTF explanation to cache")?;
    
    print_explanation(&explanation);
    
    Ok(())
}

fn generate_fingerprint(error_text: &str) -> String {
    // Normalize the error text for fingerprinting
    let normalized = normalize_error_text(error_text);
    
    // Hash the normalized text
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let hash = hasher.finalize();
    
    // Return hex string
    format!("{:x}", hash)
}

fn normalize_error_text(text: &str) -> String {
    // Normalize: lowercase, remove extra whitespace, remove file paths, remove line numbers
    let mut normalized = text.to_lowercase();
    
    // Remove common file paths (simplified)
    if let Ok(re) = regex::Regex::new(r"/[^\s]+\.(rs|py|js|ts|go|java|cpp|c|h)") {
        normalized = re.replace_all(&normalized, "<file>").to_string();
    }
    
    // Remove line numbers
    if let Ok(re) = regex::Regex::new(r":\d+:\d+") {
        normalized = re.replace_all(&normalized, ":<line>").to_string();
    }
    
    // Remove extra whitespace
    if let Ok(re) = regex::Regex::new(r"\s+") {
        normalized = re.replace_all(&normalized, " ").to_string();
    }
    
    normalized.trim().to_string()
}

fn generate_explanation(error_text: &str, fingerprint: &str) -> WtfExplanation {
    // Deterministic explanation stub based on error patterns
    let (cause, suggested_fix) = analyze_error_pattern(error_text);
    
    WtfExplanation {
        fingerprint: fingerprint.to_string(),
        cause,
        suggested_fix,
        confidence: 0.5, // Static confidence for now
    }
}

fn analyze_error_pattern(error_text: &str) -> (String, String) {
    let lower = error_text.to_lowercase();
    
    // Pattern matching for common errors
    if lower.contains("cannot find module") || lower.contains("module not found") {
        (
            "Module or file not found".to_string(),
            "Check the file path and ensure the module exists. Verify imports and file structure.".to_string(),
        )
    } else if lower.contains("undefined") || lower.contains("is not defined") {
        (
            "Variable or function is undefined".to_string(),
            "Ensure the variable/function is declared before use. Check for typos and scope issues.".to_string(),
        )
    } else if lower.contains("permission denied") || lower.contains("access denied") {
        (
            "Permission denied".to_string(),
            "Check file permissions. Try running with appropriate permissions or fix file ownership.".to_string(),
        )
    } else if lower.contains("syntax error") || lower.contains("parse error") {
        (
            "Syntax or parse error".to_string(),
            "Check for missing brackets, quotes, or semicolons. Verify the syntax matches the language requirements.".to_string(),
        )
    } else if lower.contains("type error") || lower.contains("type mismatch") {
        (
            "Type mismatch or type error".to_string(),
            "Check that variable types match expected types. Verify function signatures and return types.".to_string(),
        )
    } else if lower.contains("connection refused") || lower.contains("connection failed") {
        (
            "Connection failed".to_string(),
            "Check if the service is running. Verify network connectivity and firewall settings.".to_string(),
        )
    } else if lower.contains("out of memory") || lower.contains("memory") {
        (
            "Memory issue".to_string(),
            "Check for memory leaks or excessive memory usage. Consider optimizing data structures or increasing available memory.".to_string(),
        )
    } else {
        (
            "Unknown error pattern".to_string(),
            "Review the error message carefully. Check documentation, logs, and related code for clues.".to_string(),
        )
    }
}

fn print_explanation(explanation: &WtfExplanation) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    writeln!(handle, "Error Explanation").unwrap();
    writeln!(handle, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").unwrap();
    writeln!(handle, "Cause: {}", explanation.cause).unwrap();
    writeln!(handle, "Suggested Fix: {}", explanation.suggested_fix).unwrap();
    writeln!(handle, "Confidence: {:.0}%", explanation.confidence * 100.0).unwrap();
}
