use crate::app::AppContext;
use crate::cli::{CtxArgs, CtxCommand};
use crate::state::store::{Context, ProjectType};
use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;

/// Run the context switching command
pub fn run(args: CtxArgs, app_ctx: &mut AppContext) -> Result<()> {
    match args.command {
        CtxCommand::Save { name } => save_context(name, app_ctx),
        CtxCommand::Switch { name } => switch_context(name, app_ctx),
        CtxCommand::List => list_contexts(app_ctx),
        CtxCommand::Show { name } => show_context(name, app_ctx),
        CtxCommand::Delete { name } => delete_context(name, app_ctx),
        CtxCommand::Diff { name } => diff_context(name, app_ctx),
    }
}

fn save_context(name: String, app_ctx: &mut AppContext) -> Result<()> {
    println!("Detecting project context...");
    
    let context = capture_context(&name)?;
    
    app_ctx.state.save_context(context)
        .with_context(|| format!("Failed to save context '{}'", name))?;
    
    println!("✓ Saved context '{}'", name);
    Ok(())
}

fn capture_context(name: &str) -> Result<Context> {
    // Get current working directory
    let working_dir = env::current_dir()
        .context("Failed to get current working directory")?
        .to_string_lossy()
        .to_string();
    
    let working_path = Path::new(&working_dir);
    
    // Get git branch if in a git repo
    let git_branch = get_git_branch().ok();
    
    // Get environment variables (whitelist common ones)
    let env_vars = get_env_vars();
    
    // Detect project type and dependencies
    let (project_type, dependencies, package_manager) = detect_project(working_path)?;
    
    // Detect virtual environment
    let virtual_env = detect_virtual_env(working_path)?;
    
    // Find important files and directories
    let important_files = find_important_files(working_path, &project_type)?;
    let important_dirs = find_important_dirs(working_path, &project_type)?;
    
    Ok(Context {
        name: name.to_string(),
        working_dir,
        git_branch,
        env_vars,
        on_switch: None,
        project_type,
        dependencies,
        virtual_env,
        important_files,
        important_dirs,
        package_manager,
    })
}

fn diff_context(name: String, app_ctx: &mut AppContext) -> Result<()> {
    // 1. Get saved context
    let saved_ctx = app_ctx.state.get_context(&name)
        .with_context(|| format!("Failed to get context '{}'", name))?
        .ok_or_else(|| anyhow::anyhow!("Context '{}' not found", name))?;
        
    // 2. Capture current state
    let current_ctx = capture_context("current")?;
    
    println!("Context Diff: Current vs Saved ('{}')\n", name);
    
    // Working Directory
    if current_ctx.working_dir != saved_ctx.working_dir {
        println!("📂 Working Dircetory:");
        println!("  Current: {}", current_ctx.working_dir);
        println!("  Saved:   {}", saved_ctx.working_dir);
    } else {
        println!("📂 Working Directory: Unchanged ({})", saved_ctx.working_dir);
    }
    
    // Git Branch
    if current_ctx.git_branch != saved_ctx.git_branch {
        println!("\n🌿 Git Branch:");
        println!("  Current: {}", current_ctx.git_branch.as_deref().unwrap_or("None"));
        println!("  Saved:   {}", saved_ctx.git_branch.as_deref().unwrap_or("None"));
        
        // Check for uncommitted changes if we go to a different branch or same branch
        if let Ok(changes) = check_uncommitted_changes(Path::new(&current_ctx.working_dir)) {
            if changes > 0 {
                println!("  ⚠️  {} uncommitted changes in current directory", changes);
            }
        }
    } else {
        println!("\n🌿 Git Branch: Unchanged ({})", saved_ctx.git_branch.as_deref().unwrap_or("None"));
        if let Ok(changes) = check_uncommitted_changes(Path::new(&current_ctx.working_dir)) {
            if changes > 0 {
                println!("  ⚠️  {} uncommitted changes", changes);
            }
        }
    }
    
    // Dependencies
    // Compare dependencies maps
    let mut added_deps = Vec::new();
    let mut removed_deps = Vec::new();
    
    for (dep, _) in &current_ctx.dependencies {
        if !saved_ctx.dependencies.contains_key(dep) {
            added_deps.push(dep);
        }
    }
    
    for (dep, _) in &saved_ctx.dependencies {
        if !current_ctx.dependencies.contains_key(dep) {
            removed_deps.push(dep);
        }
    }
    
    if !added_deps.is_empty() || !removed_deps.is_empty() {
        println!("\n📚 Dependencies:");
        for dep in added_deps {
            println!("  + {}", dep);
        }
        for dep in removed_deps {
            println!("  - {}", dep);
        }
    } else {
        println!("\n📚 Dependencies: Unchanged");
    }
    
    // Env Vars
    // Basic count difference or specific vars
    let mut different_vars = Vec::new();
    for (k, v) in &current_ctx.env_vars {
        if let Some(saved_v) = saved_ctx.env_vars.get(k) {
            if v != saved_v {
                different_vars.push(k);
            }
        } else {
            different_vars.push(k); // New var
        }
    }
    
    if !different_vars.is_empty() {
        println!("\n🔐 Environment Variables:");
        println!("  {} variables differ or are new", different_vars.len());
    } else {
        println!("\n🔐 Environment Variables: Unchanged");
    }

    Ok(())
}

fn check_uncommitted_changes(working_dir: &Path) -> Result<usize> {
    let output = Command::new("git")
        .current_dir(working_dir)
        .arg("status")
        .arg("--porcelain")
        .output()
        .context("Failed to check git status")?;
        
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let changes = stdout.lines().count();
        Ok(changes)
    } else {
        Ok(0) // Not a git repo or error
    }
}

fn switch_context(name: String, app_ctx: &mut AppContext) -> Result<()> {
    let context = app_ctx.state.get_context(&name)
        .with_context(|| format!("Failed to get context '{}'", name))?
        .ok_or_else(|| anyhow::anyhow!("Context '{}' not found", name))?;
    
    // Output shell-executable code
    // This is designed to be eval'd: eval "$(dev ctx switch demo)"
    let shell = crate::platform::detect_shell().unwrap_or(crate::platform::Shell::Bash);
    
    match shell {
        crate::platform::Shell::Bash | crate::platform::Shell::Zsh => {
            output_bash_switch(&context);
        }
        crate::platform::Shell::Fish => {
            output_fish_switch(&context);
        }
        crate::platform::Shell::PowerShell => {
            output_powershell_switch(&context);
        }
        _ => {
            output_bash_switch(&context);
        }
    }
    
    Ok(())
}

fn output_bash_switch(context: &Context) {
    // Change directory
    println!("cd {}", escape_shell(&context.working_dir));
    
    // Set environment variables
    for (key, value) in &context.env_vars {
        println!("export {}={}", key, escape_shell(value));
    }
    
    // Checkout git branch if applicable
    if let Some(branch) = &context.git_branch {
        println!("git checkout {} 2>/dev/null || true", escape_shell(branch));
    }
    
    // Activate virtual environment if present
    if let Some(venv) = &context.virtual_env {
        let venv_path = Path::new(&context.working_dir).join(venv);
        if venv_path.join("bin/activate").exists() {
            println!("source {}", escape_shell(&venv_path.join("bin/activate").to_string_lossy()));
        } else if venv_path.join("Scripts/activate").exists() {
            println!("source {}", escape_shell(&venv_path.join("Scripts/activate").to_string_lossy()));
        }
    }
    
    // Install dependencies if needed (check if node_modules/cargo.lock exists)
    if let Some(pm) = &context.package_manager {
        match pm.as_str() {
            "npm" | "yarn" | "pnpm" => {
                let node_modules = Path::new(&context.working_dir).join("node_modules");
                if !node_modules.exists() {
                    println!("echo 'Installing dependencies...'");
                    if pm == "yarn" {
                        println!("yarn install 2>/dev/null || npm install");
                    } else if pm == "pnpm" {
                        println!("pnpm install 2>/dev/null || npm install");
                    } else {
                        println!("npm install");
                    }
                }
            }
            "cargo" => {
                // Cargo dependencies are managed automatically
            }
            "pip" | "pip3" => {
                if let Some(req_file) = context.dependencies.get("requirements") {
                    let req_path = Path::new(&context.working_dir).join(req_file);
                    if req_path.exists() {
                        println!("echo 'Installing Python dependencies...'");
                        println!("{} install -r {} 2>/dev/null || true", pm, escape_shell(&req_path.to_string_lossy()));
                    }
                }
            }
            _ => {}
        }
    }
    
    // Run on_switch hook if present
    if let Some(hook) = &context.on_switch {
        println!("{}", hook);
    }
}

fn output_fish_switch(context: &Context) {
    println!("cd {}", escape_shell(&context.working_dir));
    for (key, value) in &context.env_vars {
        println!("set -gx {} {}", key, escape_shell(value));
    }
    if let Some(branch) = &context.git_branch {
        println!("git checkout {} 2>/dev/null; or true", escape_shell(branch));
    }
    if let Some(venv) = &context.virtual_env {
        let venv_path = Path::new(&context.working_dir).join(venv);
        if venv_path.join("bin/activate.fish").exists() {
            println!("source {}", escape_shell(&venv_path.join("bin/activate.fish").to_string_lossy()));
        }
    }
    if let Some(hook) = &context.on_switch {
        println!("{}", hook);
    }
}

fn output_powershell_switch(context: &Context) {
    println!("Set-Location {}", escape_powershell(&context.working_dir));
    for (key, value) in &context.env_vars {
        println!("$env:{} = {}", key, escape_powershell(value));
    }
    if let Some(branch) = &context.git_branch {
        println!("git checkout {} 2>$null; if ($?) {{ }}", escape_powershell(branch));
    }
    if let Some(venv) = &context.virtual_env {
        let venv_path = Path::new(&context.working_dir).join(venv);
        if venv_path.join("Scripts/Activate.ps1").exists() {
            println!("& {}", escape_powershell(&venv_path.join("Scripts/Activate.ps1").to_string_lossy()));
        }
    }
    if let Some(hook) = &context.on_switch {
        println!("{}", hook);
    }
}

fn list_contexts(app_ctx: &mut AppContext) -> Result<()> {
    let contexts = app_ctx.state.list_contexts()
        .context("Failed to list contexts")?;
    
    if contexts.is_empty() {
        println!("No contexts saved.");
        println!("\nTo save a context, run: dev ctx save <name>");
    } else {
        println!("Saved contexts:");
        for name in contexts {
            // Get context details for better display
            if let Ok(Some(ctx)) = app_ctx.state.get_context(&name) {
                let project_info = if let Some(pt) = &ctx.project_type {
                    format!(" ({:?})", pt)
                } else {
                    String::new()
                };
                println!("  • {}{}", name, project_info);
            } else {
                println!("  • {}", name);
            }
        }
    }
    
    Ok(())
}

fn show_context(name: String, app_ctx: &mut AppContext) -> Result<()> {
    let context = app_ctx.state.get_context(&name)
        .with_context(|| format!("Failed to get context '{}'", name))?
        .ok_or_else(|| anyhow::anyhow!("Context '{}' not found", name))?;
    
    println!("Context: {}\n", context.name);
    println!("{}", "━".repeat(50));
    
    // Basic Information
    println!("\n📁 Working Directory:");
    println!("  {}", context.working_dir);
    
    if let Some(branch) = &context.git_branch {
        println!("\n🌿 Git Branch:");
        println!("  {}", branch);
    }
    
    // Project Type
    if let Some(pt) = &context.project_type {
        println!("\n📦 Project Type:");
        println!("  {:?}", pt);
    }
    
    // Package Manager
    if let Some(pm) = &context.package_manager {
        println!("\n🔧 Package Manager:");
        println!("  {}", pm);
    }
    
    // Virtual Environment
    if let Some(venv) = &context.virtual_env {
        println!("\n🐍 Virtual Environment:");
        println!("  {}", venv);
    }
    
    // Dependencies
    if !context.dependencies.is_empty() {
        println!("\n📚 Dependencies:");
        for (key, value) in &context.dependencies {
            println!("  • {}: {}", key, value);
        }
    }
    
    // Important Files
    if !context.important_files.is_empty() {
        println!("\n📄 Important Files:");
        for file in &context.important_files {
            println!("  • {}", file);
        }
    }
    
    // Important Directories
    if !context.important_dirs.is_empty() {
        println!("\n📂 Important Directories:");
        for dir in &context.important_dirs {
            println!("  • {}", dir);
        }
    }
    
    // Environment Variables
    if !context.env_vars.is_empty() {
        println!("\n🔐 Environment Variables:");
        for (key, value) in &context.env_vars {
            // Truncate long values for display
            let display_value = if value.len() > 60 {
                format!("{}...", &value[..60])
            } else {
                value.clone()
            };
            println!("  • {} = {}", key, display_value);
        }
    }
    
    // On Switch Hook
    if let Some(hook) = &context.on_switch {
        println!("\n⚡ On Switch Hook:");
        println!("  {}", hook);
    }
    
    println!("\n{}", "━".repeat(50));
    
    Ok(())
}

fn delete_context(name: String, app_ctx: &mut AppContext) -> Result<()> {
    app_ctx.state.delete_context(&name)
        .with_context(|| format!("Failed to delete context '{}'", name))?;
    
    println!("✓ Deleted context '{}'", name);
    Ok(())
}

fn detect_project(working_dir: &Path) -> Result<(Option<ProjectType>, HashMap<String, String>, Option<String>)> {
    let mut dependencies = HashMap::new();
    let mut package_manager = None;
    let mut project_type = None;
    
    // Check for Rust project
    if working_dir.join("Cargo.toml").exists() {
        project_type = Some(ProjectType::Rust);
        package_manager = Some("cargo".to_string());
        dependencies.insert("Cargo.toml".to_string(), "Cargo.toml".to_string());
        if working_dir.join("Cargo.lock").exists() {
            dependencies.insert("Cargo.lock".to_string(), "Cargo.lock".to_string());
        }
    }
    // Check for Node.js project
    else if working_dir.join("package.json").exists() {
        project_type = Some(ProjectType::Node);
        dependencies.insert("package.json".to_string(), "package.json".to_string());
        
        // Detect package manager
        if working_dir.join("yarn.lock").exists() {
            package_manager = Some("yarn".to_string());
            dependencies.insert("yarn.lock".to_string(), "yarn.lock".to_string());
        } else if working_dir.join("pnpm-lock.yaml").exists() {
            package_manager = Some("pnpm".to_string());
            dependencies.insert("pnpm-lock.yaml".to_string(), "pnpm-lock.yaml".to_string());
        } else {
            package_manager = Some("npm".to_string());
        }
        
        if working_dir.join("package-lock.json").exists() {
            dependencies.insert("package-lock.json".to_string(), "package-lock.json".to_string());
        }
    }
    // Check for Python project
    else if working_dir.join("requirements.txt").exists() || 
            working_dir.join("pyproject.toml").exists() ||
            working_dir.join("setup.py").exists() ||
            working_dir.join("Pipfile").exists() {
        project_type = Some(ProjectType::Python);
        
        if working_dir.join("requirements.txt").exists() {
            dependencies.insert("requirements".to_string(), "requirements.txt".to_string());
        }
        if working_dir.join("pyproject.toml").exists() {
            dependencies.insert("pyproject.toml".to_string(), "pyproject.toml".to_string());
        }
        if working_dir.join("Pipfile").exists() {
            dependencies.insert("Pipfile".to_string(), "Pipfile".to_string());
            package_manager = Some("pipenv".to_string());
        } else {
            package_manager = Some("pip3".to_string());
        }
    }
    // Check for Go project
    else if working_dir.join("go.mod").exists() {
        project_type = Some(ProjectType::Go);
        package_manager = Some("go".to_string());
        dependencies.insert("go.mod".to_string(), "go.mod".to_string());
        if working_dir.join("go.sum").exists() {
            dependencies.insert("go.sum".to_string(), "go.sum".to_string());
        }
    }
    // Check for Java project
    else if working_dir.join("pom.xml").exists() || working_dir.join("build.gradle").exists() {
        project_type = Some(ProjectType::Java);
        if working_dir.join("pom.xml").exists() {
            dependencies.insert("pom.xml".to_string(), "pom.xml".to_string());
            package_manager = Some("maven".to_string());
        }
        if working_dir.join("build.gradle").exists() {
            dependencies.insert("build.gradle".to_string(), "build.gradle".to_string());
            package_manager = Some("gradle".to_string());
        }
    }
    
    Ok((project_type, dependencies, package_manager))
}

fn detect_virtual_env(working_dir: &Path) -> Result<Option<String>> {
    // Check for common virtual environment directories
    let venv_dirs = vec![
        "venv", ".venv", "env", ".env", "virtualenv",
        "node_modules/.bin", // Node virtual env equivalent
    ];
    
    for venv_dir in venv_dirs {
        let venv_path = working_dir.join(venv_dir);
        if venv_path.exists() {
            // Check if it's actually a virtual environment
            if venv_path.join("bin/activate").exists() ||
               venv_path.join("Scripts/activate").exists() ||
               venv_path.join("Scripts/Activate.ps1").exists() ||
               venv_dir.starts_with("node_modules") {
                return Ok(Some(venv_dir.to_string()));
            }
        }
    }
    
    // Check parent directories for virtual environments
    if let Some(parent) = working_dir.parent() {
        for venv_dir in &["venv", ".venv", "env", ".env"] {
            let venv_path = parent.join(venv_dir);
            if venv_path.exists() {
                if venv_path.join("bin/activate").exists() ||
                   venv_path.join("Scripts/activate").exists() {
                    return Ok(Some(format!("../{}", venv_dir)));
                }
            }
        }
    }
    
    Ok(None)
}

fn find_important_files(working_dir: &Path, project_type: &Option<ProjectType>) -> Result<Vec<String>> {
    let mut files = Vec::new();
    
    // Common important files
    let common_files = vec![
        ".gitignore", ".env", ".env.local", "README.md", "LICENSE",
        "Makefile", "Dockerfile", ".dockerignore",
    ];
    
    for file in common_files {
        if working_dir.join(file).exists() {
            files.push(file.to_string());
        }
    }
    
    // Project-specific files
    if let Some(pt) = project_type {
        match pt {
            ProjectType::Rust => {
                if working_dir.join("rustfmt.toml").exists() {
                    files.push("rustfmt.toml".to_string());
                }
                if working_dir.join(".rustfmt.toml").exists() {
                    files.push(".rustfmt.toml".to_string());
                }
            }
            ProjectType::Node => {
                if working_dir.join(".nvmrc").exists() {
                    files.push(".nvmrc".to_string());
                }
                if working_dir.join(".node-version").exists() {
                    files.push(".node-version".to_string());
                }
            }
            ProjectType::Python => {
                if working_dir.join(".python-version").exists() {
                    files.push(".python-version".to_string());
                }
            }
            _ => {}
        }
    }
    
    Ok(files)
}

fn find_important_dirs(working_dir: &Path, project_type: &Option<ProjectType>) -> Result<Vec<String>> {
    let mut dirs = Vec::new();
    
    // Common important directories
    let common_dirs = vec!["src", "lib", "tests", "test", "docs", "doc", "config", "scripts"];
    
    for dir in common_dirs {
        if working_dir.join(dir).is_dir() {
            dirs.push(dir.to_string());
        }
    }
    
    // Project-specific directories
    if let Some(pt) = project_type {
        match pt {
            ProjectType::Rust => {
                if working_dir.join("examples").is_dir() {
                    dirs.push("examples".to_string());
                }
                if working_dir.join("benches").is_dir() {
                    dirs.push("benches".to_string());
                }
            }
            ProjectType::Node => {
                if working_dir.join("public").is_dir() {
                    dirs.push("public".to_string());
                }
                if working_dir.join("components").is_dir() {
                    dirs.push("components".to_string());
                }
            }
            ProjectType::Python => {
                if working_dir.join("app").is_dir() {
                    dirs.push("app".to_string());
                }
            }
            _ => {}
        }
    }
    
    Ok(dirs)
}

fn get_git_branch() -> Result<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .context("Failed to run git command")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Not in a git repository"));
    }
    
    let branch = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();
    
    Ok(branch)
}

fn get_env_vars() -> HashMap<String, String> {
    // Whitelist of common environment variables to save
    let whitelist = vec![
        "PATH", "HOME", "USER", "SHELL", "EDITOR", "LANG", "LC_ALL",
        "GOPATH", "GOROOT", "JAVA_HOME", "PYTHONPATH", "NODE_PATH",
        "RUST_BACKTRACE", "CARGO_HOME", "RUSTUP_HOME",
    ];
    
    let mut env_vars = HashMap::new();
    for key in whitelist {
        if let Ok(value) = env::var(key) {
            env_vars.insert(key.to_string(), value);
        }
    }
    
    env_vars
}

fn escape_shell(s: &str) -> String {
    // Simple shell escaping - wrap in single quotes and escape single quotes
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

fn escape_powershell(s: &str) -> String {
    // PowerShell escaping - wrap in single quotes
    format!("'{}'", s.replace('\'', "''"))
}
