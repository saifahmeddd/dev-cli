use clap::{Parser, Subcommand};

/// A stateful developer workflow CLI tool
#[derive(Parser, Debug)]
#[command(name = "dev")]
#[command(about = "A stateful developer workflow CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Context switching commands
    Ctx(CtxArgs),
    /// Error explanation (wtf)
    Wtf(WtfArgs),
    /// Command snippet management
    Snip(SnipArgs),
    /// Configuration management
    Config(ConfigArgs),
}

#[derive(Parser, Debug)]
pub struct CtxArgs {
    #[command(subcommand)]
    pub command: CtxCommand,
}

#[derive(Subcommand, Debug)]
pub enum CtxCommand {
    /// Save current context
    Save {
        /// Name of the context
        name: String,
    },
    /// Switch to a saved context (outputs shell code)
    Switch {
        /// Name of the context
        name: String,
    },
    /// List all saved contexts
    List,
    /// Delete a saved context
    Delete {
        /// Name of the context
        name: String,
    },
}

#[derive(Parser, Debug)]
pub struct WtfArgs {
    /// Error text to explain
    #[arg(required = true)]
    pub error_text: String,
}

#[derive(Parser, Debug)]
pub struct SnipArgs {
    #[command(subcommand)]
    pub command: SnipCommand,
}

#[derive(Subcommand, Debug)]
pub enum SnipCommand {
    /// Add a new snippet
    Add {
        /// Name of the snippet
        name: String,
        /// Command template (use {var} for variables)
        command: String,
        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// List all snippets
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Run a snippet
    Run {
        /// Name of the snippet
        name: String,
        /// Variables as key=value pairs
        #[arg(short, long)]
        vars: Vec<String>,
        /// Dry-run mode (don't execute, just show command)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    // Placeholder for future arguments
}

