use std::env;
use std::collections::HashMap;
use anyhow::Result;

pub fn get_current_env() -> HashMap<String, String> {
    env::vars().collect()
}

pub fn apply_env(vars: &HashMap<String, String>) {
    for (key, value) in vars {
        env::set_var(key, value);
    }
}

pub fn diff_env(old: &HashMap<String, String>, new: &HashMap<String, String>) -> HashMap<String, String> {
    let mut diff = HashMap::new();
    for (key, value) in new {
        if let Some(old_val) = old.get(key) {
            if old_val != value {
                diff.insert(key.clone(), value.clone());
            }
        } else {
             diff.insert(key.clone(), value.clone());
        }
    }
    diff
}
