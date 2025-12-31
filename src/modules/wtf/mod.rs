use crate::cli::WtfArgs;
use anyhow::Result;
use console::style;

pub mod capture;
pub mod api;
pub mod cache;

pub async fn handle_wtf(args: &WtfArgs) -> Result<()> {
    let error_msg = if let Some(msg) = &args.error_msg {
        msg.clone()
    } else {
        match capture::capture_last_error() {
            Ok(msg) => msg,
            Err(e) => {
                println!("{}", style(format!("Info: {}", e)).yellow());
                return Ok(());
            }
        }
    };

    println!("{}", style("Analyzing error...").cyan());

    // Check cache
    let mut cache = cache::ErrorCache::load()?;
    if let Some(explanation) = cache.get(&error_msg) {
        println!("{}", style("Found cached explanation:").green());
        println!("{}", explanation);
        return Ok(());
    }

    // Call API
    println!("{}", style("Asking Claude...").dim());
    match api::explain_error(&error_msg).await {
        Ok(explanation) => {
            println!("\n{}", style("Explanation:").bold());
            println!("{}", explanation);
            
            // Save to cache
            cache.insert(error_msg, explanation);
            if let Err(e) = cache.save() {
                eprintln!("Failed to save cache: {}", e);
            }
        }
        Err(e) => {
             println!("{}", style(format!("Failed to get explanation: {}", e)).red());
        }
    }

    Ok(())
}
