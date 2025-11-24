mod client;
mod prompt;
mod enricher;

pub use enricher::TaskEnricher;

use serde::{Deserialize, Serialize};

/// Enriched task data parsed from natural language input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedTask {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl EnrichedTask {
    /// Create a simple task with just a title (fallback when LLM unavailable)
    pub fn simple(title: String) -> Self {
        Self {
            title,
            due_date: None,
            priority: None,
            tags: Vec::new(),
            context: None,
        }
    }
}
