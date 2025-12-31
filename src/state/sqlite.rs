use crate::state::store::{Context, Snippet, StateStore, WtfExplanation};
use anyhow::{Context as AnyhowContext, Result};
use rusqlite::{params, Connection};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

/// SQLite-backed implementation of StateStore
/// Uses RefCell since this is a single-threaded CLI application
pub struct SqliteStore {
    conn: RefCell<Connection>,
}

impl SqliteStore {
    /// Create a new SqliteStore instance
    pub fn new() -> Result<Self> {
        let db_path = get_db_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory for database: {:?}", parent))?;
        }
        
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;
        
        Ok(SqliteStore {
            conn: RefCell::new(conn),
        })
    }
}

fn get_db_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .context("Could not determine home directory")?;
    Ok(home.join(".dev").join("state.db"))
}

impl StateStore for SqliteStore {
    fn init(&mut self) -> Result<()> {
        let conn = self.conn.borrow_mut();
        
        // Create contexts table with migration support
        conn.execute(
            "CREATE TABLE IF NOT EXISTS contexts (
                name TEXT PRIMARY KEY,
                working_dir TEXT NOT NULL,
                git_branch TEXT,
                env_vars TEXT NOT NULL,
                on_switch TEXT,
                project_type TEXT,
                dependencies TEXT,
                virtual_env TEXT,
                important_files TEXT,
                important_dirs TEXT,
                package_manager TEXT
            )",
            [],
        )?;
        
        // Migrate existing contexts table if needed (ignore errors if columns already exist)
        // SQLite doesn't support IF NOT EXISTS for ALTER TABLE, so we check first
        let table_info_result: Result<Vec<String>, _> = conn
            .prepare("PRAGMA table_info(contexts)")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| Ok(row.get::<_, String>(1)?))
                    .and_then(|iter| iter.collect::<Result<Vec<_>, _>>())
            });
        
        if let Ok(table_info) = table_info_result {
            if !table_info.contains(&"project_type".to_string()) {
                let _ = conn.execute("ALTER TABLE contexts ADD COLUMN project_type TEXT", []);
            }
            if !table_info.contains(&"dependencies".to_string()) {
                // SQLite doesn't support DEFAULT in ALTER TABLE, so we add without it
                let _ = conn.execute("ALTER TABLE contexts ADD COLUMN dependencies TEXT", []);
            }
            if !table_info.contains(&"virtual_env".to_string()) {
                let _ = conn.execute("ALTER TABLE contexts ADD COLUMN virtual_env TEXT", []);
            }
            if !table_info.contains(&"important_files".to_string()) {
                let _ = conn.execute("ALTER TABLE contexts ADD COLUMN important_files TEXT", []);
            }
            if !table_info.contains(&"important_dirs".to_string()) {
                let _ = conn.execute("ALTER TABLE contexts ADD COLUMN important_dirs TEXT", []);
            }
            if !table_info.contains(&"package_manager".to_string()) {
                let _ = conn.execute("ALTER TABLE contexts ADD COLUMN package_manager TEXT", []);
            }
        }
        
        // Create wtf_explanations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wtf_explanations (
                fingerprint TEXT PRIMARY KEY,
                cause TEXT NOT NULL,
                suggested_fix TEXT NOT NULL,
                confidence REAL NOT NULL
            )",
            [],
        )?;
        
        // Create snippets table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS snippets (
                name TEXT PRIMARY KEY,
                command TEXT NOT NULL,
                tags TEXT NOT NULL
            )",
            [],
        )?;
        
        Ok(())
    }
    
    fn close(&mut self) -> Result<()> {
        // SQLite connection closes automatically when dropped
        Ok(())
    }
    
    fn save_context(&mut self, context: Context) -> Result<()> {
        let env_vars_json = serde_json::to_string(&context.env_vars)?;
        let dependencies_json = serde_json::to_string(&context.dependencies)?;
        let important_files_json = serde_json::to_string(&context.important_files)?;
        let important_dirs_json = serde_json::to_string(&context.important_dirs)?;
        let project_type_json = context.project_type.as_ref()
            .map(|pt| serde_json::to_string(pt).unwrap_or_default());
        
        let conn = self.conn.borrow_mut();
        
        conn.execute(
            "INSERT OR REPLACE INTO contexts (
                name, working_dir, git_branch, env_vars, on_switch,
                project_type, dependencies, virtual_env, important_files, important_dirs, package_manager
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                context.name,
                context.working_dir,
                context.git_branch,
                env_vars_json,
                context.on_switch,
                project_type_json,
                dependencies_json,
                context.virtual_env,
                important_files_json,
                important_dirs_json,
                context.package_manager
            ],
        )?;
        
        Ok(())
    }
    
    fn get_context(&self, name: &str) -> Result<Option<Context>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT name, working_dir, git_branch, env_vars, on_switch,
                    project_type, dependencies, virtual_env, important_files, important_dirs, package_manager
             FROM contexts WHERE name = ?1"
        )?;
        
        let mut rows = stmt.query_map(params![name], |row| {
            let env_vars_json: String = row.get(3)?;
            let env_vars: HashMap<String, String> = serde_json::from_str(&env_vars_json)
                .unwrap_or_default();
            
            let project_type_json: Option<String> = row.get(5)?;
            let project_type = project_type_json
                .and_then(|json| serde_json::from_str(&json).ok());
            
            let dependencies_json: String = row.get(6).unwrap_or_else(|_| "{}".to_string());
            let dependencies: HashMap<String, String> = serde_json::from_str(&dependencies_json)
                .unwrap_or_default();
            
            let important_files_json: String = row.get(8).unwrap_or_else(|_| "[]".to_string());
            let important_files: Vec<String> = serde_json::from_str(&important_files_json)
                .unwrap_or_default();
            
            let important_dirs_json: String = row.get(9).unwrap_or_else(|_| "[]".to_string());
            let important_dirs: Vec<String> = serde_json::from_str(&important_dirs_json)
                .unwrap_or_default();
            
            Ok(Context {
                name: row.get(0)?,
                working_dir: row.get(1)?,
                git_branch: row.get(2)?,
                env_vars,
                on_switch: row.get(4)?,
                project_type,
                dependencies,
                virtual_env: row.get(7)?,
                important_files,
                important_dirs,
                package_manager: row.get(10)?,
            })
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
    
    fn list_contexts(&self) -> Result<Vec<String>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare("SELECT name FROM contexts ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut names = Vec::new();
        for row in rows {
            names.push(row?);
        }
        
        Ok(names)
    }
    
    fn delete_context(&mut self, name: &str) -> Result<()> {
        let conn = self.conn.borrow_mut();
        conn.execute("DELETE FROM contexts WHERE name = ?1", params![name])?;
        Ok(())
    }
    
    fn get_wtf_explanation(&self, fingerprint: &str) -> Result<Option<WtfExplanation>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT fingerprint, cause, suggested_fix, confidence
             FROM wtf_explanations WHERE fingerprint = ?1"
        )?;
        
        let mut rows = stmt.query_map(params![fingerprint], |row| {
            Ok(WtfExplanation {
                fingerprint: row.get(0)?,
                cause: row.get(1)?,
                suggested_fix: row.get(2)?,
                confidence: row.get(3)?,
            })
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
    
    fn save_wtf_explanation(&mut self, explanation: WtfExplanation) -> Result<()> {
        let conn = self.conn.borrow_mut();
        conn.execute(
            "INSERT OR REPLACE INTO wtf_explanations (fingerprint, cause, suggested_fix, confidence)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                explanation.fingerprint,
                explanation.cause,
                explanation.suggested_fix,
                explanation.confidence
            ],
        )?;
        
        Ok(())
    }
    
    fn save_snippet(&mut self, snippet: Snippet) -> Result<()> {
        let tags_json = serde_json::to_string(&snippet.tags)?;
        let conn = self.conn.borrow_mut();
        
        conn.execute(
            "INSERT OR REPLACE INTO snippets (name, command, tags)
             VALUES (?1, ?2, ?3)",
            params![snippet.name, snippet.command, tags_json],
        )?;
        
        Ok(())
    }
    
    fn get_snippet(&self, name: &str) -> Result<Option<Snippet>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT name, command, tags FROM snippets WHERE name = ?1"
        )?;
        
        let mut rows = stmt.query_map(params![name], |row| {
            let tags_json: String = row.get(2)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json)
                .unwrap_or_default();
            
            Ok(Snippet {
                name: row.get(0)?,
                command: row.get(1)?,
                tags,
            })
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
    
    fn list_snippets(&self, tag_filter: Option<&str>) -> Result<Vec<Snippet>> {
        let conn = self.conn.borrow();
        
        let mut snippets = Vec::new();
        
        if let Some(tag) = tag_filter {
            let tag_pattern = format!("%\"{}\"%", tag);
            let mut stmt = conn.prepare(
                "SELECT name, command, tags FROM snippets WHERE tags LIKE ?1 ORDER BY name"
            )?;
            
            let rows = stmt.query_map(params![tag_pattern], |row| {
                let tags_json: String = row.get(2)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json)
                    .unwrap_or_default();
                
                Ok(Snippet {
                    name: row.get(0)?,
                    command: row.get(1)?,
                    tags,
                })
            })?;
            
            for row in rows {
                snippets.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT name, command, tags FROM snippets ORDER BY name"
            )?;
            
            let rows = stmt.query_map([], |row| {
                let tags_json: String = row.get(2)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json)
                    .unwrap_or_default();
                
                Ok(Snippet {
                    name: row.get(0)?,
                    command: row.get(1)?,
                    tags,
                })
            })?;
            
            for row in rows {
                snippets.push(row?);
            }
        }
        
        Ok(snippets)
    }
    
    fn delete_snippet(&mut self, name: &str) -> Result<()> {
        let conn = self.conn.borrow_mut();
        conn.execute("DELETE FROM snippets WHERE name = ?1", params![name])?;
        Ok(())
    }
}

