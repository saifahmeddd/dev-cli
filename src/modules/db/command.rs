use crate::app::AppContext;
use crate::cli::{DbArgs, DbCommand};
use anyhow::Result;
use std::process::Command;

pub fn run(args: DbArgs, _app_ctx: &mut AppContext) -> Result<()> {
    match args.command {
        DbCommand::List => list_dbs(),
        DbCommand::Connect { name } => connect(&name),
        DbCommand::Backup { name } => backup(&name),
        DbCommand::Migrate => migrate(),
        DbCommand::Seed => seed(),
        DbCommand::Diff => diff(),
    }
}

fn list_dbs() -> Result<()> {
    println!("🗄️  Configured Databases:");
    println!("  - local (sqlite)");
    println!("  - staging (postgres)");
    println!("  (This is a placeholder. Future, configure in dev.toml)");
    Ok(())
}

fn connect(name: &str) -> Result<()> {
    println!("🔌 Connecting to '{}'...", name);
    if name == "local" {
        Command::new("sqlite3").arg("dev.db").status()?;
    } else if name == "staging" {
        println!("Assume postgres...");
         // Command::new("psql")... 
         println!("(Simulation) Connected to postgres.");
    } else {
        println!("Unknown database '{}'", name);
    }
    Ok(())
}

fn backup(name: &str) -> Result<()> {
     println!("💾 Backing up '{}'...", name);
     // simplistic backup
     if name == "local" {
         println!("Running sqlite3 .dump...");
     }
     Ok(())
}

fn migrate() -> Result<()> {
    println!("🏗️  Running migrations...");
    if std::path::Path::new("sqlx-data.json").exists() {
         println!("Detected sqlx project. Running sqlx migrate run...");
         Command::new("sqlx").arg("migrate").arg("run").status()?;
    } else {
        println!("No migration system detected.");
    }
    Ok(())
}

fn seed() -> Result<()> {
    println!("🌱 Seeding database...");
    Ok(())
}

fn diff() -> Result<()> {
    println!("⚖️  Comparing database schemas...");
    Ok(())
}
