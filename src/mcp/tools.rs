use crate::models::{ItemType, Priority, Status, TaskFilter, TaskItem};
use crate::storage::Storage;
use serde_json::{json, Value};

/// Handle initialize request
pub fn initialize() -> Result<Value, String> {
    Ok(json!({
        "protocolVersion": "0.1.0",
        "serverInfo": {
            "name": "tasktui",
            "version": "0.1.0"
        },
        "capabilities": {
            "tools": true,
            "resources": true
        }
    }))
}

/// List available tools
pub fn list_tools() -> Result<Value, String> {
    Ok(json!({
        "tools": [
            {
                "name": "create_task",
                "description": "Create a new task with optional metadata",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Task title"
                        },
                        "context": {
                            "type": "string",
                            "description": "Task context/notes"
                        },
                        "due_date": {
                            "type": "string",
                            "description": "Due date in YYYY-MM-DD format"
                        },
                        "priority": {
                            "type": "string",
                            "enum": ["low", "medium", "high"],
                            "description": "Task priority"
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Task tags"
                        }
                    },
                    "required": ["title"]
                }
            },
            {
                "name": "update_task",
                "description": "Update a task field or append notes",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task UUID"
                        },
                        "field": {
                            "type": "string",
                            "enum": ["title", "status", "priority", "tags", "due_date", "notes"],
                            "description": "Field to update"
                        },
                        "value": {
                            "description": "New value"
                        }
                    },
                    "required": ["id", "field", "value"]
                }
            },
            {
                "name": "list_tasks",
                "description": "List tasks with optional filtering",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["active", "next", "waiting", "done", "archived"],
                            "description": "Filter by status"
                        },
                        "tag": {
                            "type": "string",
                            "description": "Filter by tag"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results"
                        }
                    }
                }
            },
            {
                "name": "read_task_details",
                "description": "Get full details of a specific task",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task UUID"
                        }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "complete_task",
                "description": "Mark a task as done",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Task UUID"
                        }
                    },
                    "required": ["id"]
                }
            }
        ]
    }))
}

/// Call a tool
pub fn call_tool(storage: &Storage, params: Value) -> Result<Value, String> {
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing tool name")?;

    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

    match tool_name {
        "create_task" => create_task(storage, arguments),
        "update_task" => update_task(storage, arguments),
        "list_tasks" => list_tasks(storage, arguments),
        "read_task_details" => read_task_details(storage, arguments),
        "complete_task" => complete_task(storage, arguments),
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

fn create_task(storage: &Storage, args: Value) -> Result<Value, String> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or("Missing title")?
        .to_string();

    let mut task = TaskItem::new(title, ItemType::Task);

    if let Some(context) = args.get("context").and_then(|v| v.as_str()) {
        task.body = context.to_string();
    }

    if let Some(due_date) = args.get("due_date").and_then(|v| v.as_str()) {
        task.frontmatter.due_date = Some(due_date.to_string());
    }

    if let Some(priority) = args.get("priority").and_then(|v| v.as_str()) {
        task.frontmatter.priority = match priority {
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium,
        };
    }

    if let Some(tags) = args.get("tags").and_then(|v| v.as_array()) {
        task.frontmatter.tags = tags
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }

    storage
        .write_task(&task)
        .map_err(|e| format!("Failed to write task: {}", e))?;

    Ok(json!({
        "id": task.frontmatter.id,
        "title": task.frontmatter.title,
        "status": "created"
    }))
}

fn update_task(storage: &Storage, args: Value) -> Result<Value, String> {
    let id_str = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing id")?;

    let id = uuid::Uuid::parse_str(id_str).map_err(|e| format!("Invalid UUID: {}", e))?;

    let field = args
        .get("field")
        .and_then(|v| v.as_str())
        .ok_or("Missing field")?;

    let value = args.get("value").ok_or("Missing value")?;

    let mut tasks = storage
        .load_all_tasks()
        .map_err(|e| format!("Failed to load tasks: {}", e))?;

    let task = tasks
        .iter_mut()
        .find(|t| t.frontmatter.id == id)
        .ok_or("Task not found")?;

    match field {
        "title" => {
            task.frontmatter.title = value.as_str().ok_or("Invalid title")?.to_string();
        }
        "status" => {
            let status_str = value.as_str().ok_or("Invalid status")?;
            task.frontmatter.status = match status_str {
                "active" => Status::Active,
                "next" => Status::Next,
                "waiting" => Status::Waiting,
                "done" => Status::Done,
                "archived" => Status::Archived,
                _ => return Err("Invalid status value".to_string()),
            };
        }
        "priority" => {
            let priority_str = value.as_str().ok_or("Invalid priority")?;
            task.frontmatter.priority = match priority_str {
                "high" => Priority::High,
                "medium" => Priority::Medium,
                "low" => Priority::Low,
                _ => return Err("Invalid priority value".to_string()),
            };
        }
        "notes" => {
            let notes = value.as_str().ok_or("Invalid notes")?;
            task.body.push_str("\n\n");
            task.body.push_str(notes);
        }
        _ => return Err(format!("Unknown field: {}", field)),
    }

    storage
        .write_task(task)
        .map_err(|e| format!("Failed to write task: {}", e))?;

    Ok(json!({ "status": "updated" }))
}

fn list_tasks(storage: &Storage, args: Value) -> Result<Value, String> {
    let mut filter = TaskFilter::default();

    if let Some(status_str) = args.get("status").and_then(|v| v.as_str()) {
        filter.status = Some(match status_str {
            "active" => Status::Active,
            "next" => Status::Next,
            "waiting" => Status::Waiting,
            "done" => Status::Done,
            "archived" => Status::Archived,
            _ => return Err("Invalid status".to_string()),
        });
    }

    if let Some(tag) = args.get("tag").and_then(|v| v.as_str()) {
        filter.tags.push(tag.to_string());
    }

    if let Some(limit) = args.get("limit").and_then(|v| v.as_u64()) {
        filter.limit = Some(limit as usize);
    }

    let tasks = storage
        .list_tasks(&filter)
        .map_err(|e| format!("Failed to list tasks: {}", e))?;

    let task_list: Vec<Value> = tasks
        .iter()
        .map(|task| {
            json!({
                "id": task.frontmatter.id,
                "title": task.frontmatter.title,
                "status": task.frontmatter.status.as_str(),
                "priority": match task.frontmatter.priority {
                    Priority::High => "high",
                    Priority::Medium => "medium",
                    Priority::Low => "low",
                },
                "tags": task.frontmatter.tags,
                "due_date": task.frontmatter.due_date,
            })
        })
        .collect();

    Ok(json!({ "tasks": task_list }))
}

fn read_task_details(storage: &Storage, args: Value) -> Result<Value, String> {
    let id_str = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing id")?;

    let id = uuid::Uuid::parse_str(id_str).map_err(|e| format!("Invalid UUID: {}", e))?;

    let tasks = storage
        .load_all_tasks()
        .map_err(|e| format!("Failed to load tasks: {}", e))?;

    let task = tasks
        .iter()
        .find(|t| t.frontmatter.id == id)
        .ok_or("Task not found")?;

    Ok(json!({
        "id": task.frontmatter.id,
        "title": task.frontmatter.title,
        "type": match task.frontmatter.item_type {
            ItemType::Task => "task",
            ItemType::Goal => "goal",
            ItemType::Note => "note",
        },
        "status": task.frontmatter.status.as_str(),
        "priority": match task.frontmatter.priority {
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        },
        "tags": task.frontmatter.tags,
        "due_date": task.frontmatter.due_date,
        "created_at": task.frontmatter.created_at,
        "body": task.body,
    }))
}

fn complete_task(storage: &Storage, args: Value) -> Result<Value, String> {
    let id_str = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing id")?;

    let id = uuid::Uuid::parse_str(id_str).map_err(|e| format!("Invalid UUID: {}", e))?;

    let mut tasks = storage
        .load_all_tasks()
        .map_err(|e| format!("Failed to load tasks: {}", e))?;

    let task = tasks
        .iter_mut()
        .find(|t| t.frontmatter.id == id)
        .ok_or("Task not found")?;

    task.frontmatter.status = Status::Done;

    storage
        .write_task(task)
        .map_err(|e| format!("Failed to write task: {}", e))?;

    Ok(json!({ "status": "completed" }))
}

/// List available resources
pub fn list_resources() -> Result<Value, String> {
    Ok(json!({
        "resources": [
            {
                "uri": "tasktui://daily_summary",
                "name": "Daily Summary",
                "description": "A summary of today's high-priority tasks",
                "mimeType": "application/json"
            }
        ]
    }))
}

/// Read a resource
pub fn read_resource(storage: &Storage, params: Value) -> Result<Value, String> {
    let uri = params
        .get("uri")
        .and_then(|v| v.as_str())
        .ok_or("Missing uri")?;

    match uri {
        "tasktui://daily_summary" => daily_summary(storage),
        _ => Err(format!("Unknown resource: {}", uri)),
    }
}

fn daily_summary(storage: &Storage) -> Result<Value, String> {
    let mut filter = TaskFilter::default();
    filter.status = Some(Status::Active);
    filter.limit = Some(10);

    let tasks = storage
        .list_tasks(&filter)
        .map_err(|e| format!("Failed to list tasks: {}", e))?;

    let high_priority: Vec<_> = tasks
        .iter()
        .filter(|t| t.frontmatter.priority == Priority::High)
        .collect();

    let due_today: Vec<_> = tasks.iter().filter(|t| t.is_due_today()).collect();

    Ok(json!({
        "summary": {
            "total_active": tasks.len(),
            "high_priority_count": high_priority.len(),
            "due_today_count": due_today.len(),
            "high_priority_tasks": high_priority.iter().map(|t| {
                json!({
                    "id": t.frontmatter.id,
                    "title": t.frontmatter.title,
                    "tags": t.frontmatter.tags,
                })
            }).collect::<Vec<_>>(),
            "due_today_tasks": due_today.iter().map(|t| {
                json!({
                    "id": t.frontmatter.id,
                    "title": t.frontmatter.title,
                    "tags": t.frontmatter.tags,
                })
            }).collect::<Vec<_>>(),
        }
    }))
}
