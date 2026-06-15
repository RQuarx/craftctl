use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceProfile {
    pub java_runtime: String,
    pub memory_min_mb: u16,
    pub memory_max_mb: u16,
}

impl Default for InstanceProfile {
    fn default() -> Self {
        Self {
            java_runtime: "system".to_string(),
            memory_min_mb: 1024,
            memory_max_mb: 4096,
        }
    }
}
