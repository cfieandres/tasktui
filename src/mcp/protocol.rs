use crate::storage::Storage;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

use super::tools;

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub struct McpServer {
    storage: Storage,
}

impl McpServer {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        eprintln!("MCP Server started. Listening on stdio...");

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            eprintln!("Received: {}", line);

            let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => self.handle_request(request),
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                },
            };

            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }

    fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => tools::initialize(),
            "tools/list" => tools::list_tools(),
            "tools/call" => {
                let params = request.params.unwrap_or(Value::Null);
                tools::call_tool(&self.storage, params)
            }
            "resources/list" => tools::list_resources(),
            "resources/read" => {
                let params = request.params.unwrap_or(Value::Null);
                tools::read_resource(&self.storage, params)
            }
            _ => Err(format!("Method not found: {}", request.method)),
        };

        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: e,
                    data: None,
                }),
            },
        }
    }
}
