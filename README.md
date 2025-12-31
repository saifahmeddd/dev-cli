# dev - A Stateful Developer Workflow CLI

A production-grade, cross-platform developer CLI tool written in Rust that helps you manage project contexts, explain errors, and manage command snippets. Built for developers who work across multiple projects and want to maintain context and workflow efficiency.

## Features

### 🎯 Context Switching (`dev ctx`)
- **Project-Oriented Context Management**: Automatically detects and saves project details
- **Auto-Detection**: Detects project type (Rust, Node.js, Python, Go, Java), dependencies, virtual environments, and important files
- **Smart Restoration**: Restores working directory, git branch, environment variables, and activates virtual environments
- **Cross-Platform**: Works on macOS, Linux, and Windows (PowerShell)

### 🤔 Error Explanation (`dev wtf`)
- **Error Fingerprinting**: Uses SHA256 hashing to identify similar errors
- **Local Caching**: Stores explanations in SQLite for fast retrieval
- **Pattern Matching**: Provides deterministic explanations for common error patterns
- **Confidence Levels**: Shows confidence scores for explanations

### 📝 Snippet Management (`dev snip`)
- **Variable Substitution**: Use `{variable}` syntax in snippets
- **Tagging System**: Organize snippets with tags
- **Dry-Run Mode**: Test snippets before execution
- **Cross-Platform Execution**: Runs snippets via system shell

## Installation

### Prerequisites
- Rust (stable version)
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone or navigate to the project directory
cd /path/to/dev-cli

# Build the release binary
cargo build --release

# Install globally
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/dev` (make sure `~/.cargo/bin` is in your PATH).

### Verify Installation

```bash
dev --help
```

## Quick Start

### 1. Save Your First Context

Navigate to any project directory and save its context:

```bash
cd ~/projects/my-awesome-project
dev ctx save my-project
```

The tool automatically detects:
- Project type (Rust, Node.js, Python, etc.)
- Dependencies (package.json, Cargo.toml, requirements.txt, etc.)
- Virtual environments
- Important files and directories
- Git branch
- Environment variables

### 2. View Saved Contexts

```bash
dev ctx list
```

### 3. Show Context Details

```bash
dev ctx show my-project
```

### 4. Switch to a Context

From anywhere in your terminal:

**Bash/Zsh:**
```bash
eval "$(dev ctx switch my-project)"
```

**Fish:**
```bash
eval (dev ctx switch my-project | psub)
```

**PowerShell:**
```powershell
dev ctx switch my-project | Invoke-Expression
```

This will:
- Change to the project directory
- Restore environment variables
- Checkout the saved git branch
- Activate virtual environment (if detected)
- Install dependencies if missing

## Command Reference

### Context Commands (`dev ctx`)

#### `dev ctx save <name>`
Saves the current project context.

**What it captures:**
- Working directory
- Git branch (if in a git repository)
- Project type and dependencies
- Virtual environment
- Important files and directories
- Environment variables (whitelisted)

**Example:**
```bash
dev ctx save frontend-app
```

#### `dev ctx switch <name>`
Outputs shell-executable code to restore a saved context.

**Usage:**
```bash
# Bash/Zsh
eval "$(dev ctx switch frontend-app)"

# Fish
eval (dev ctx switch frontend-app | psub)

# PowerShell
dev ctx switch frontend-app | Invoke-Expression
```

**What it restores:**
- Changes to the saved directory
- Sets environment variables
- Checks out git branch
- Activates virtual environment
- Installs dependencies if needed

#### `dev ctx list`
Lists all saved contexts with their project types.

**Example output:**
```
Saved contexts:
  • frontend-app (Node)
  • backend-api (Rust)
  • mlops-project (Python)
```

#### `dev ctx show <name>`
Displays detailed information about a saved context.

**Example:**
```bash
dev ctx show frontend-app
```

**Output includes:**
- Working directory
- Git branch
- Project type
- Package manager
- Virtual environment
- Dependencies
- Important files and directories
- Environment variables
- On-switch hooks

#### `dev ctx delete <name>`
Deletes a saved context.

**Example:**
```bash
dev ctx delete old-project
```

### Error Explanation (`dev wtf`)

#### `dev wtf "<error text>"`
Explains an error message.

**Features:**
- Error fingerprinting (normalizes and hashes error text)
- Local caching in SQLite
- Pattern matching for common errors
- Confidence scores

**Example:**
```bash
dev wtf "cannot find module 'express'"
```

**Output:**
```
Error Explanation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Cause: Module or file not found
Suggested Fix: Check the file path and ensure the module exists. Verify imports and file structure.
Confidence: 50%
```

**Supported Error Patterns:**
- Module not found
- Undefined variables/functions
- Permission denied
- Syntax errors
- Type errors
- Connection failures
- Memory issues

### Snippet Management (`dev snip`)

#### `dev snip add <name> <command> [--tags <tags>]`
Adds a new command snippet.

**Example:**
```bash
dev snip add deploy "docker build -t {image} . && docker push {image}" --tags docker,deploy
```

#### `dev snip list [--tag <tag>]`
Lists all snippets, optionally filtered by tag.

**Example:**
```bash
dev snip list
dev snip list --tag docker
```

#### `dev snip run <name> [--vars <key=value>...] [--dry-run]`
Runs a snippet with variable substitution.

**Example:**
```bash
# Dry run (preview)
dev snip run deploy --vars image=myapp:v1.0 --dry-run

# Execute
dev snip run deploy --vars image=myapp:v1.0
```

**Variable Substitution:**
Use `{variable}` syntax in snippets:
```bash
dev snip add greet "echo Hello, {name}!"
dev snip run greet --vars name=World
# Output: Hello, World!
```

## Configuration

### Config File Location

The tool looks for configuration at:
- `~/.dev/config.toml` (user config)
- Falls back to bundled `config/default.toml`

### State Storage

All state (contexts, wtf explanations, snippets) is stored in:
- `~/.dev/state.db` (SQLite database)

## Project Structure

```
dev/
├── Cargo.toml
├── src/
│   ├── main.rs                # Entry point
│   ├── cli.rs                 # CLI definitions
│   ├── app.rs                 # Command dispatcher
│   ├── config/                # Configuration management
│   │   ├── mod.rs
│   │   ├── load.rs
│   │   └── model.rs
│   ├── state/                 # State storage
│   │   ├── mod.rs
│   │   ├── store.rs           # StateStore trait
│   │   └── sqlite.rs          # SQLite implementation
│   ├── modules/               # Feature modules
│   │   ├── ctx/              # Context switching
│   │   ├── wtf/               # Error explanation
│   │   └── snip/              # Snippet management
│   └── platform/              # Platform abstraction
│       └── shell.rs           # Shell detection
└── config/
    └── default.toml           # Default configuration
```

## Architecture

### Design Principles

1. **Separation of Concerns**: Clear module boundaries
2. **Config vs State**: Config = user intent (TOML), State = learned behavior (SQLite)
3. **No Global State**: All state passed through AppContext
4. **Trait-Based Abstractions**: StateStore trait allows different implementations
5. **Cross-Platform**: Works on macOS, Linux, and Windows

### State Management

- **SQLite Database**: Stores all persistent state
- **Automatic Migration**: Handles schema updates automatically
- **Backward Compatible**: Old contexts continue to work after updates

## Examples

### Example 1: Managing Multiple Projects

```bash
# Save contexts for different projects
cd ~/projects/frontend && dev ctx save frontend
cd ~/projects/backend && dev ctx save backend
cd ~/projects/api && dev ctx save api

# List all contexts
dev ctx list

# Switch between projects
eval "$(dev ctx switch frontend)"
# Work on frontend...

eval "$(dev ctx switch backend)"
# Work on backend...
```

### Example 2: Error Troubleshooting

```bash
# Get explanation for an error
dev wtf "ModuleNotFoundError: No module named 'requests'"

# The explanation is cached, so subsequent queries are instant
dev wtf "ModuleNotFoundError: No module named 'requests'"
```

### Example 3: Command Snippets

```bash
# Add a snippet for deploying
dev snip add deploy "docker build -t {image}:{tag} . && docker push {image}:{tag}"

# Add a snippet for testing
dev snip add test "npm test -- {test_file}" --tags testing

# List all snippets
dev snip list

# Run with variables
dev snip run deploy --vars image=myapp tag=v1.0.0
dev snip run test --vars test_file=user.test.js
```

### Example 4: Shell Integration

Add to your `~/.zshrc` or `~/.bashrc`:

```bash
# Quick context switching
dev-switch() {
    eval "$(dev ctx switch $1)"
}

# Alias for convenience
alias dcs='dev ctx switch'
alias dcl='dev ctx list'
alias dcsave='dev ctx save'
```

Then use:
```bash
dev-switch my-project
# or
dcs my-project
```

## Troubleshooting

### Command Not Found

If `dev` command is not found:
1. Ensure `~/.cargo/bin` is in your PATH
2. Add to `~/.zshrc` or `~/.bashrc`: `export PATH="$PATH:$HOME/.cargo/bin"`
3. Reload shell: `source ~/.zshrc`

### Context Not Switching

- Make sure you're using `eval "$(...)"` for bash/zsh
- Check that the context exists: `dev ctx list`
- Verify context details: `dev ctx show <name>`

### Database Issues

If you encounter database errors:
- The database is at `~/.dev/state.db`
- You can delete it to start fresh (will lose all saved data)
- The tool will recreate it automatically

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Running Locally

```bash
cargo run -- ctx list
cargo run -- ctx save test
cargo run -- ctx show test
```

## Requirements

- Rust 1.70+ (stable)
- SQLite (bundled with rusqlite)
- Git (for git branch detection)

## License

[Add your license here]

## Contributing

[Add contributing guidelines here]

## Roadmap

- [ ] Shell hooks for automatic context detection
- [ ] LLM integration for better error explanations
- [ ] Team sync for sharing contexts
- [ ] Learning heuristics for workflow optimization
- [ ] Background process management
- [ ] More project type detections

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - CLI framework
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling
- [serde](https://github.com/serde-rs/serde) - Serialization

---

**Made with ❤️ for developers who juggle multiple projects**

