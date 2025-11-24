/// System prompt for task enrichment
pub const SYSTEM_PROMPT: &str = r#"You are a task parsing assistant. Your job is to extract structured information from natural language task descriptions.

Given a task description, extract:
1. **title**: A clean, concise task title (required)
2. **due_date**: Date in YYYY-MM-DD format if mentioned (e.g., "tomorrow", "next monday", "dec 25")
3. **priority**: One of "high", "medium", "low" if urgency is indicated
4. **tags**: Any categories or contexts mentioned (e.g., "work", "personal", "home", "shopping")
5. **context**: Additional notes or details that don't fit in other fields

Examples:
- "call mom tomorrow" → title: "Call mom", due_date: "{tomorrow}", priority: null, tags: ["personal"]
- "urgent meeting prep for work" → title: "Meeting prep", priority: "high", tags: ["work"]
- "buy groceries this weekend low priority" → title: "Buy groceries", due_date: "{weekend}", priority: "low", tags: ["shopping"]
- "finish the report" → title: "Finish the report", (no other fields)

Respond ONLY with valid JSON matching this schema:
{
  "title": "string (required)",
  "due_date": "string YYYY-MM-DD or null",
  "priority": "high|medium|low or null",
  "tags": ["array", "of", "strings"],
  "context": "string or null"
}

Today's date is: {today}"#;

/// Build the user prompt with the raw input
pub fn build_user_prompt(raw_input: &str) -> String {
    format!("Parse this task: \"{}\"", raw_input)
}

/// Build the system prompt with today's date
pub fn build_system_prompt(today: &str) -> String {
    SYSTEM_PROMPT.replace("{today}", today)
        .replace("{tomorrow}", &calculate_date_offset(today, 1))
        .replace("{weekend}", &calculate_next_weekend(today))
}

/// Calculate a date offset from today
fn calculate_date_offset(today: &str, days: i64) -> String {
    use chrono::{NaiveDate, Duration};

    if let Ok(date) = NaiveDate::parse_from_str(today, "%Y-%m-%d") {
        (date + Duration::days(days)).format("%Y-%m-%d").to_string()
    } else {
        today.to_string()
    }
}

/// Calculate the next Saturday from today
fn calculate_next_weekend(today: &str) -> String {
    use chrono::{NaiveDate, Datelike, Duration, Weekday};

    if let Ok(date) = NaiveDate::parse_from_str(today, "%Y-%m-%d") {
        let days_until_saturday = (Weekday::Sat.num_days_from_monday() as i64
            - date.weekday().num_days_from_monday() as i64 + 7) % 7;
        let days = if days_until_saturday == 0 { 7 } else { days_until_saturday };
        (date + Duration::days(days)).format("%Y-%m-%d").to_string()
    } else {
        today.to_string()
    }
}
