use crate::app::AppContext;
use crate::cli::{SecretsArgs, SecretsCommand};
use anyhow::Result;
use keyring::Entry;
use dialoguer::{theme::ColorfulTheme, Password};

pub fn run(args: SecretsArgs, _app_ctx: &mut AppContext) -> Result<()> {
    match args.command {
        SecretsCommand::Add { key, value } => add_secret(&key, value),
        SecretsCommand::Get { key } => get_secret(&key),
        SecretsCommand::List => list_secrets(),
        SecretsCommand::Rotate { key } => rotate_secret(&key),
        SecretsCommand::Sync => {
            println!("Secrets sync not implemented.");
            Ok(())
        }
    }
}

fn add_secret(key: &str, value: Option<String>) -> Result<()> {
    let secret = match value {
        Some(v) => v,
        None => {
            Password::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Enter secret for '{}'", key))
                .interact()?
        }
    };
    
    let entry = Entry::new("dev-cli", key)?;
    entry.set_password(&secret)?;
    println!("✅ Secret '{}' stored securey.", key);
    Ok(())
}

fn get_secret(key: &str) -> Result<()> {
    let entry = Entry::new("dev-cli", key)?;
    match entry.get_password() {
        Ok(secret) => {
            println!("🔒 Secret '{}': {}", key, secret);
            // In a real app we might verify if user really wants to see it or copy to clipboard
        },
        Err(e) => println!("❌ Secret '{}' error: {}", key, e),
    }
    Ok(())
}

fn list_secrets() -> Result<()> {
    println!("🔑 Stored Secrets (Keys only):");
    // keyring crate generic interface doesn't easily support listing all keys for a service 
    // across all platforms comfortably without platform specific code.
    // For this MVP, we might not be able to list them easily unless we store the keys list in our sqlite db.
    
    println!("  (Listing keys is platform dependent and not fully supported in this version. Check your system keychain for 'dev-cli' entries.)");
    Ok(())
}

fn rotate_secret(key: &str) -> Result<()> {
    // Generate new random value
    use rand::{distributions::Alphanumeric, Rng};
    let new_secret: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
        
    let entry = Entry::new("dev-cli", key)?;
    entry.set_password(&new_secret)?;
    println!("🔄 Secret '{}' rotated. New value: {}", key, new_secret);
    Ok(())
}
