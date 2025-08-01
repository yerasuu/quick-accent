use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolConfig {
    /// Prefer focused screen over primary screen
    pub test_key: bool,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            test_key: true,
        }
    }
}
