mod protocol;
mod tools;

pub use protocol::McpServer;

use crate::storage::Storage;
use anyhow::Result;
use std::path::PathBuf;

/// Run MCP server mode
pub fn run(data_dir: PathBuf) -> Result<()> {
    let storage = Storage::new(data_dir)?;
    let server = McpServer::new(storage);
    server.run()
}
