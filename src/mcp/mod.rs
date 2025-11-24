mod protocol;
mod tools;

pub use protocol::McpServer;

use crate::config::AppConfig;
use crate::llm::TaskEnricher;
use crate::storage::Storage;
use anyhow::Result;
use std::path::PathBuf;

/// Run MCP server mode
pub fn run(data_dir: PathBuf) -> Result<()> {
    let storage = Storage::new(data_dir.clone())?;
    let config = AppConfig::load(&data_dir)?;
    let enricher = TaskEnricher::new(config.openai_api_key.clone());
    let server = McpServer::new(storage, enricher, config);
    server.run()
}
