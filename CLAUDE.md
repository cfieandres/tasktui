# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a CLI/TUI Task Manager with MCP (Model Context Protocol) integration, designed as a "Second Brain" accessible to both humans (via TUI) and AI agents (via MCP). The system is built in Rust for performance and portability.

## Core Architecture

### Dual-Head Design

The application operates in two modes from a single binary:

1. **TUI Mode** (Human): `tasktui` - Interactive terminal interface for task management
2. **MCP Server Mode** (AI): `tasktui --server` - Exposes tools/resources via stdio for AI clients

Both heads operate on the same data layer but provide different interaction patterns.

### Data Model

**Storage**: Markdown files with YAML frontmatter, one file per task/goal/note

**File naming**: `{uuid}.md`

**Frontmatter schema**:
```yaml
id: string (UUID)
type: "task" | "goal" | "note"
title: string
status: "active" | "next" | "waiting" | "done" | "archived"
priority: string
tags: array of strings
due_date: ISO date string (optional)
parent_goal_id: string (optional)
created_at: ISO timestamp
```

**Body**: Unstructured markdown for notes and context

### Auto-Sync Strategy

Every write operation triggers:
1. `git pull --rebase --autostash`
2. File modification
3. `git add . && git commit -m "Update: {task_id}" && git push`

Failed pushes set "Sync Pending" state and retry on next command.

## MCP Interface

### Tools (AI Capabilities)

- `create_task(title, context?, due_date?, priority?, notes?)` - Creates new task file
- `update_task(id, field, value)` - Patches frontmatter or appends notes
- `list_tasks(status?, tag?, limit?)` - Filtered task retrieval with JSON output
- `read_task_details(id)` - Returns full task content (frontmatter + body)
- `complete_task(id)` - Sets status to "done", optionally archives

### Resources

- `daily_summary` - Dynamic read-only snapshot of today's high-priority tasks

## Tech Stack

- **Language**: Rust (single binary, type safety, fast startup)
- **TUI**: `ratatui`
- **CLI**: `clap` (mode switching)
- **Serialization**: `serde`, `serde_yaml`
- **MCP**: `mcp-sdk-rs` or custom JSON-RPC 2.0 over stdio
- **Git**: `git2` or `std::process::Command` for git operations

## Development Commands

### Build & Run
```bash
cargo build                              # Build debug binary
cargo build --release                    # Build optimized binary
cargo run                                # Run TUI mode (default data dir: ./tasks)
cargo run -- --data-dir ~/my-tasks       # Run with custom data directory
cargo run -- server                      # Run MCP server mode
```

### Testing
```bash
cargo test                               # Run all tests
cargo test --lib                         # Run library tests only
cargo test storage::                     # Run storage tests
cargo test git::                         # Run git tests
```

### Development
```bash
cargo check                              # Fast compilation check
cargo clippy                             # Linting
cargo fmt                                # Format code
RUST_LOG=debug cargo run                 # Run with debug logging
```

### TUI Mode Shortcuts
- `Tab` - Toggle between Kanban and Compact views
- `↑↓/jk` - Navigate tasks
- `n` - New task, `d` - Done, `a` - Archive
- `1/2` - Filter by work/personal tags, `0` - Clear filters
- `r` - Refresh, `q` - Quit

## Key Implementation Considerations

### Parsing Tasks
- Use `serde_yaml` to parse frontmatter
- Separate frontmatter from markdown body at `---` delimiter
- Validate schema fields on parse, handle missing optional fields

### Filtering Logic
The MCP `list_tasks` tool must implement backend filtering to avoid sending all task content to AI. Filter by status, tags, priority, and dates before serializing results.

### Concurrent Access
Since both TUI and MCP server can run simultaneously, implement file locking or advisory locks when writing to prevent corruption.

### Error Recovery
Auto-sync failures should be non-fatal. Queue failed syncs and provide manual retry mechanism in TUI.
