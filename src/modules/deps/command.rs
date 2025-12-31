use crate::app::AppContext;
use crate::cli::{DepsArgs, DepsCommand};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(args: DepsArgs, _app_ctx: &mut AppContext) -> Result<()> {
    match args.command {
        DepsCommand::Scan => scan(),
        DepsCommand::Outdated => outdated(),
        DepsCommand::Update { safe } => update(safe),
        DepsCommand::Audit { path } => audit(&path),
        DepsCommand::Compare => compare(),
    }
}

fn scan() -> Result<()> {
    println!("🔍 Scanning for vulnerabilities...");
    
    if Path::new("Cargo.toml").exists() {
        println!("🦀 Detected Rust project");
        // Check if cargo-audit is installed
        let check = Command::new("cargo").arg("audit").arg("--version").output();
        if let Ok(output) = check {
            if output.status.success() {
                let status = Command::new("cargo").arg("audit").status()?;
                 if !status.success() {
                    println!("⚠️  Vulnerabilities found!");
                } else {
                    println!("✅ No vulnerabilities found.");
                }
            } else {
                 println!("⚠️  cargo-audit not installed. Run `cargo install cargo-audit`");
            }
        } else {
            println!("⚠️  cargo-audit not installed. Run `cargo install cargo-audit`");
        }
    }
    
    if Path::new("package.json").exists() {
        println!("📦 Detected Node.js project");
        let status = Command::new("npm").arg("audit").status()?;
        if !status.success() {
            println!("⚠️  Vulnerabilities found!");
        } else {
            println!("✅ No vulnerabilities found.");
        }
    }
    
    Ok(())
}

fn outdated() -> Result<()> {
    println!("🕰️  Checking for outdated packages...");
    
    if Path::new("Cargo.toml").exists() {
        // cargo outdated
        let check = Command::new("cargo").arg("outdated").arg("--version").output();
        if let Ok(output) = check {
             if output.status.success() {
                Command::new("cargo").arg("outdated").status()?;
             } else {
                 println!("⚠️  cargo-outdated not installed. Run `cargo install cargo-outdated`");
             }
        } else {
             println!("⚠️  cargo-outdated not installed. Run `cargo install cargo-outdated`");
        }
    }
    
    if Path::new("package.json").exists() {
        Command::new("npm").arg("outdated").status()?;
    }
    
    Ok(())
}

fn update(safe: bool) -> Result<()> {
    println!("⬆️  Updating dependencies...");
    
    if Path::new("Cargo.toml").exists() {
        if safe {
            // Rust doesn't have a direct "safe only" update like npm strictly,
            // but `cargo update` usually updates to latest compatible (semver).
            println!("Running cargo update...");
            Command::new("cargo").arg("update").status()?;
        } else {
            println!("Running cargo update...");
            Command::new("cargo").arg("update").status()?;
        }
    }
    
    if Path::new("package.json").exists() {
        if safe {
            println!("Running npm update...");
             Command::new("npm").arg("update").status()?;
        } else {
             println!("To update major versions, consider using `npm install <pkg>@latest` or `ncu`");
             Command::new("npm").arg("update").status()?;
        }
    }
    
    Ok(())
}

fn audit(_path: &str) -> Result<()> {
    // Recursive audit placeholder
    println!("Auditing directory tree... (Recursive implementation would go here)");
    // Reuse scan() logic for each dir
    scan()
}

fn compare() -> Result<()> {
    println!("Dependency comparison not yet implemented.");
    Ok(())
}
