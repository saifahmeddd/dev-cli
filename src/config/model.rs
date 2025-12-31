use serde::Deserialize;

/// User configuration representing user intent
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Placeholder fields for future features
    // These will be populated as features are added
}

impl Default for Config {
    fn default() -> Self {
        Config {}
    }
}

