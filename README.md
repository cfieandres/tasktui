# TaskTUI - Terminal Task Manager with MCP

A high-performance, terminal-based task manager designed for power users and AI agents. Features a beautiful TUI with dual view modes and full MCP (Model Context Protocol) integration for AI assistants.

## Features

- **Dual-Head Architecture**: Run as interactive TUI or MCP server
- **Two View Modes**: Toggle between Kanban board and Compact list views
- **Dark/Yellow Theme**: Beautiful cyberpunk-inspired aesthetic
- **Auto-Sync**: Automatic git pull-commit-push on every change
- **Markdown Storage**: Tasks stored as markdown files with YAML frontmatter
- **MCP Integration**: Full JSON-RPC 2.0 server for AI assistants

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/tasktui` (or `tasktui.exe` on Windows).

## Usage

### TUI Mode (Human Interface)

```bash
# Run with default data directory (./tasks)
tasktui

# Specify custom data directory
tasktui --data-dir ~/my-tasks
```

#### Keyboard Shortcuts

**Navigation:**
- `↑/k` - Move up
- `↓/j` - Move down
- `Tab` - Toggle between Kanban and Compact views

**Actions:**
- `n` - Create new task
- `d` - Mark task as done
- `a` - Archive task
- `r` - Refresh tasks from disk

**Filters:**
- `1` - Filter by "work" tag
- `2` - Filter by "personal" tag
- `0` - Clear filters

**Other:**
- `q` - Quit

### MCP Server Mode (AI Interface)

```bash
tasktui --data-dir ~/tasks server
```

The server communicates via stdio using JSON-RPC 2.0 protocol.

#### Available MCP Tools

1. **create_task** - Create a new task
   - Parameters: title, context, due_date, priority, tags

2. **update_task** - Update a task field
   - Parameters: id, field, value

3. **list_tasks** - List tasks with filtering
   - Parameters: status, tag, limit

4. **read_task_details** - Get full task details
   - Parameters: id

5. **complete_task** - Mark task as done
   - Parameters: id

#### MCP Resources

- **tasktui://daily_summary** - Daily high-priority task summary

## Task File Format

Tasks are stored as markdown files with YAML frontmatter:

```markdown
---
id: "550e8400-e29b-41d4-a716-446655440000"
type: "task"
title: "Draft Q4 Strategy"
status: "active"
priority: "high"
tags: ["work", "strategy"]
due_date: "2025-11-26"
created_at: "2025-11-24T10:00:00Z"
---

## Context & Notes
Needs to include competitor analysis.
```

### Status Values
- `active` - Currently working on
- `next` - Queued for later
- `waiting` - Blocked/waiting
- `done` - Completed
- `archived` - Archived

### Priority Values
- `high` - 🔴 High priority
- `medium` - 🟠 Medium priority
- `low` - ⚪ Low priority

## Git Synchronization

If your data directory is a git repository, TaskTUI automatically:

1. Pulls with rebase before writing
2. Commits changes after writing
3. Pushes to remote

To set up git sync:

```bash
cd tasks
git init
git remote add origin <your-repo-url>
git add .
git commit -m "Initial tasks"
git push -u origin main
```

## Development

```bash
# Run in debug mode
cargo run

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Build optimized binary
cargo build --release
```

## Architecture

- **models.rs** - Task data structures and frontmatter schema
- **storage.rs** - File I/O and task persistence
- **git.rs** - Git auto-sync functionality
- **tui/** - Terminal user interface
  - `app.rs` - Application state
  - `colors.rs` - Dark/yellow theme
  - `kanban.rs` - Kanban board view
  - `compact.rs` - Compact list view
- **mcp/** - Model Context Protocol server
  - `protocol.rs` - JSON-RPC 2.0 implementation
  - `tools.rs` - MCP tool handlers

## License

MIT
