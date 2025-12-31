use std::env;

/// Supported shell types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Unknown,
}

/// Detect the current shell from environment
pub fn detect_shell() -> Option<Shell> {
    let shell = env::var("SHELL").ok()?;
    
    if shell.contains("bash") {
        Some(Shell::Bash)
    } else if shell.contains("zsh") {
        Some(Shell::Zsh)
    } else if shell.contains("fish") {
        Some(Shell::Fish)
    } else if shell.contains("pwsh") || shell.contains("powershell") {
        Some(Shell::PowerShell)
    } else {
        Some(Shell::Unknown)
    }
}

