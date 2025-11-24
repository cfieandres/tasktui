/// System prompt for task enrichment
pub const SYSTEM_PROMPT: &str = r#"You are a GTD (Getting Things Done) task parsing assistant. Your job is to extract structured information from natural language task descriptions and rephrase them as actionable next actions.

**CRITICAL: Title must be GTD-style actionable**
- Always start with a verb (Call, Email, Review, Draft, Schedule, Buy, Fix, Update, etc.)
- Be specific and concrete about the physical next action
- Keep it concise but clear

Given a task description, extract:
1. **title**: A GTD-style actionable task title starting with a VERB (required)
   - "mom birthday" → "Call Mom to wish happy birthday" or "Buy birthday gift for Mom"
   - "report" → "Finish quarterly report" or "Review and submit report"
   - "meeting notes" → "Write up meeting notes" or "Send meeting notes to team"
2. **due_date**: Date in YYYY-MM-DD format if mentioned (e.g., "tomorrow", "next monday", "dec 25")
3. **priority**: One of "high", "medium", "low" - infer from urgency words (urgent, asap, important = high; later, whenever = low)
4. **tags**: Categories/contexts mentioned (work, personal, home, shopping, errands, etc.)
5. **context**: Additional notes that don't fit elsewhere

Examples:
- "call mom tomorrow" → title: "Call Mom", due_date: "{tomorrow}", tags: ["personal"]
- "urgent meeting prep for work" → title: "Prepare materials for meeting", priority: "high", tags: ["work"]
- "buy groceries this weekend low priority" → title: "Buy groceries", due_date: "{weekend}", priority: "low", tags: ["shopping"]
- "the report" → title: "Complete the report"
- "check snowflake data" → title: "Review Snowflake data and verify accuracy"
- "email john about project" → title: "Email John regarding project status"

Respond ONLY with valid JSON:
{
  "title": "string starting with verb (required)",
  "due_date": "YYYY-MM-DD or null",
  "priority": "high|medium|low or null",
  "tags": ["array", "of", "strings"],
  "context": "string or null"
}

Today's date is: {today}"#;

/// Build the user prompt with the raw input
pub fn build_user_prompt(raw_input: &str) -> String {
    format!("Parse this task: \"{}\"", raw_input)
}

/// Build the system prompt with today's date and optional goals context
pub fn build_system_prompt(today: &str, goals_context: Option<&str>) -> String {
    let mut prompt = SYSTEM_PROMPT.replace("{today}", today)
        .replace("{tomorrow}", &calculate_date_offset(today, 1))
        .replace("{weekend}", &calculate_next_weekend(today));

    // Add goals context if available to help with prioritization
    if let Some(goals) = goals_context {
        if !goals.is_empty() {
            prompt.push_str("\n\n--- User's Goals & Priorities (GTD Horizons of Focus) ---\n");
            prompt.push_str(goals);
            prompt.push_str("\n\nUse these goals to help determine appropriate priority and tags. ");
            prompt.push_str("Tasks that align with high-priority goals should be marked as higher priority.");
        }
    }

    prompt
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
