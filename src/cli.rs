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
    /// Git workflow tools
    Git(GitArgs),
    /// Dependency management
    Deps(DepsArgs),
    /// Documentation tools
    Docs(DocsArgs),
    /// Secrets management
    Secrets(SecretsArgs),
    /// Database management
    Db(DbArgs),
}

#[derive(Parser, Debug)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: DocsCommand,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommand {
    /// Auto-generate docs from code
    Generate {
        /// File to document
        file: String,
    },
    /// Generate README.md from project structure
    Readme,
    /// Generate API documentation
    Api,
    /// Keep docs in sync with code
    Sync,
    /// Search docs
    Search {
        /// Query string
        query: String,
    },
}

#[derive(Parser, Debug)]
pub struct SecretsArgs {
    #[command(subcommand)]
    pub command: SecretsCommand,
}

#[derive(Subcommand, Debug)]
pub enum SecretsCommand {
    /// Add a new secret
    Add {
        /// Key name
        key: String,
        /// Secret value (interactive if not provided)
        #[arg(short, long)]
        value: Option<String>,
    },
    /// Retrieve a secret
    Get {
        /// Key name
        key: String,
    },
    /// List all keys
    List,
    /// Rotate a secret
    Rotate {
        /// Key name
        key: String,
    },
    /// Sync with external vault
    Sync,
}

#[derive(Parser, Debug)]
pub struct DbArgs {
    #[command(subcommand)]
    pub command: DbCommand,
}

#[derive(Subcommand, Debug)]
pub enum DbCommand {
    /// List configured databases
    List,
    /// Connect to a database shell
    Connect {
        /// Database name
        name: String,
    },
    /// Backup a database
    Backup {
        /// Database name
        name: String,
    },
    /// Run pending migrations
    Migrate,
    /// Seed database with test data
    Seed,
    /// Compare database schemas
    Diff,
}

#[derive(Parser, Debug)]
pub struct DepsArgs {
    #[command(subcommand)]
    pub command: DepsCommand,
}

#[derive(Subcommand, Debug)]
pub enum DepsCommand {
    /// Scan for security vulnerabilities
    Scan,
    /// Check for outdated packages
    Outdated,
    /// Update dependencies (wrapper)
    Update {
        /// Safe update (patches only)
        #[arg(long)]
        safe: bool,
    },
    /// Audit all projects in a directory
    Audit {
        /// Directory to audit
        #[arg(long, default_value = ".")]
        path: String,
    },
    /// Compare dependencies across projects (placeholder)
    Compare,
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
    /// Show details of a saved context
    Show {
        /// Name of the context
        name: String,
    },
    /// Delete a saved context
    Delete {
        /// Name of the context
        name: String,
    },
    /// Compare current state with a saved context
    Diff {
        /// Name of the context to compare against
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

#[derive(Parser, Debug)]
pub struct GitArgs {
    #[command(subcommand)]
    pub command: GitCommand,
}

#[derive(Subcommand, Debug)]
pub enum GitCommand {
    /// Generate a commit message using AI
    SmartCommit {
        /// Optional hint for the commit message
        #[arg(short, long)]
        hint: Option<String>,
        /// Dry run (print message but don't commit)
        #[arg(long)]
        dry_run: bool,
    },
    /// Interactive code review checklist
    Review,
    /// Clean up merged branches and stale remotes
    Cleanup {
        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
    /// Show contribution stats
    Stats,
    /// Quick worktree management
    Worktree {
        /// Name of the worktree (branch name)
        name: String,
    },
}

