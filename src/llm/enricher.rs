use super::client::OpenAIClient;
use super::prompt::{build_system_prompt, build_user_prompt};
use super::EnrichedTask;
use chrono::Utc;

pub struct TaskEnricher {
    client: Option<OpenAIClient>,
}

impl TaskEnricher {
    /// Create a new enricher with optional API key
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: api_key.map(OpenAIClient::new),
        }
    }

    /// Check if enrichment is available
    pub fn is_available(&self) -> bool {
        self.client.is_some()
    }

    /// Enrich a raw task input using LLM
    /// Falls back to simple task if LLM unavailable or fails
    pub async fn enrich(&self, raw_input: &str) -> EnrichedTask {
        // If no API key, return simple task
        let Some(client) = &self.client else {
            return EnrichedTask::simple(raw_input.to_string());
        };

        // Get today's date for the prompt
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let system_prompt = build_system_prompt(&today);
        let user_prompt = build_user_prompt(raw_input);

        // Try to get enriched response
        match client.complete(&system_prompt, &user_prompt).await {
            Ok(response) => {
                // Try to parse JSON response
                match parse_llm_response(&response) {
                    Ok(task) => task,
                    Err(_) => {
                        // Fallback: use raw input as title
                        EnrichedTask::simple(raw_input.to_string())
                    }
                }
            }
            Err(_) => {
                // API error: fallback to simple task
                EnrichedTask::simple(raw_input.to_string())
            }
        }
    }

    /// Synchronous version for non-async contexts
    /// Uses tokio runtime to block on the async call
    pub fn enrich_sync(&self, raw_input: &str) -> EnrichedTask {
        // If no API key, return simple task immediately
        if self.client.is_none() {
            return EnrichedTask::simple(raw_input.to_string());
        }

        // Try to get or create a tokio runtime
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're already in an async context, spawn blocking
                std::thread::scope(|s| {
                    s.spawn(|| {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(self.enrich(raw_input))
                    }).join().unwrap_or_else(|_| EnrichedTask::simple(raw_input.to_string()))
                })
            }
            Err(_) => {
                // No runtime, create one
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt.block_on(self.enrich(raw_input)),
                    Err(_) => EnrichedTask::simple(raw_input.to_string()),
                }
            }
        }
    }
}

/// Parse the LLM JSON response into an EnrichedTask
fn parse_llm_response(response: &str) -> Result<EnrichedTask, String> {
    // Try to find JSON in the response (it might have markdown code blocks)
    let json_str = extract_json(response)?;

    serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON parse error: {}", e))
}

/// Extract JSON from a response that might have markdown formatting
fn extract_json(response: &str) -> Result<String, String> {
    let trimmed = response.trim();

    // If it starts with {, assume it's raw JSON
    if trimmed.starts_with('{') {
        return Ok(trimmed.to_string());
    }

    // Try to find JSON in code blocks
    if let Some(start) = trimmed.find("```json") {
        let after_marker = &trimmed[start + 7..];
        if let Some(end) = after_marker.find("```") {
            return Ok(after_marker[..end].trim().to_string());
        }
    }

    // Try generic code block
    if let Some(start) = trimmed.find("```") {
        let after_marker = &trimmed[start + 3..];
        // Skip optional language identifier
        let content_start = after_marker.find('\n').unwrap_or(0);
        let after_newline = &after_marker[content_start..];
        if let Some(end) = after_newline.find("```") {
            return Ok(after_newline[..end].trim().to_string());
        }
    }

    // Try to find { and } directly
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start < end {
            return Ok(trimmed[start..=end].to_string());
        }
    }

    Err("No JSON found in response".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_raw() {
        let response = r#"{"title": "Test", "tags": []}"#;
        assert!(extract_json(response).is_ok());
    }

    #[test]
    fn test_extract_json_code_block() {
        let response = r#"```json
{"title": "Test", "tags": []}
```"#;
        assert!(extract_json(response).is_ok());
    }

    #[test]
    fn test_parse_llm_response() {
        let response = r#"{"title": "Call mom", "due_date": "2024-12-25", "priority": "high", "tags": ["personal"], "context": null}"#;
        let task = parse_llm_response(response).unwrap();
        assert_eq!(task.title, "Call mom");
        assert_eq!(task.due_date, Some("2024-12-25".to_string()));
        assert_eq!(task.priority, Some("high".to_string()));
        assert_eq!(task.tags, vec!["personal"]);
    }
}
