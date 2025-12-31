use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "dev")]
#[command(about = "The Ultimate Developer CLI Tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Project Context Switcher
    Ctx(CtxArgs),
    /// Error Explainer
    Wtf(WtfArgs),
    /// Command Snippet Manager
    Snip(SnipArgs),
    /// Tool configuration
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
pub struct CtxArgs {
    #[command(subcommand)]
    pub command: Option<CtxCommands>,
}

#[derive(Subcommand, Debug)]
pub enum CtxCommands {
    Switch { name: String },
    Save { name: String },
    Show { name: String },
    List,
    Current,
    Delete { name: String },
}

#[derive(Args, Debug)]
pub struct WtfArgs {
    pub error_msg: Option<String>,
}

#[derive(Args, Debug)]
pub struct SnipArgs {
    #[command(subcommand)]
    pub command: Option<SnipCommands>,
}

#[derive(Subcommand, Debug)]
pub enum SnipCommands {
    Add,
    Run {
        #[arg(default_value = None)]
        name: Option<String>,
        #[arg(last = true)]
        args: Vec<String>,
    },
    List,
    Search,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: Option<ConfigCommands>,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    Init,
    Edit,
    Show,
}
