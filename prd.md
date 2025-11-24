### **Product Requirements Document (PRD): CLI/TUI Task Manager with MCP**

#### **1. Product Overview**

A high-performance, terminal-based task manager designed for power users and AI agents. It functions as a "Second Brain" that is equally accessible to a human via a TUI (Terminal User Interface) and to LLMs via the Model Context Protocol (MCP). The system prioritizes speed, structured data for AI parsing, and simplified synchronization.

#### **2. System Architecture: "Dual-Head" Design**

The application runs as a single binary with two distinct modes of operation.

  * **Head A: Interactive TUI (Human)**
      * Launched via standard command (e.g., `tasktui`).
      * Provides a visual dashboard for managing tasks, notes, and goals.
      * Immediate visual feedback.
  * **Head B: MCP Server (AI)**
      * Launched via `tasktui --server`.
      * Connects to AI clients (e.g., Claude Desktop, IDEs) via stdio.
      * Exposes tools and resources to allow the AI to read, search, filter, and edit the task database without visual rendering.

#### **3. Data Storage & Schema**

  * **Format:** Markdown files with YAML Frontmatter.
  * **Structure:** One file per task/goal, stored in a flat or date-partitioned directory.
  * **Why:** Markdown body allows for unstructured "brain dumps," while Frontmatter provides the strict schema required for filtering.

**File Template (`{uuid}.md`):**

```markdown
---
id: "550e8400-e29b-41d4-a716-446655440000"
type: "task" # or "goal", "note"
title: "Draft Q4 Strategy"
status: "active" # options: active, next, waiting, done, archived
priority: "high"
tags: ["work", "strategy"]
due_date: "2025-11-26"
parent_goal_id: "goal-123"
created_at: "2025-11-24T10:00:00Z"
---

## Context & Notes
Needs to include the analysis from the competitor review. 
Remember to check the budget spreadsheet before finalizing section 3.
```

#### **4. MCP Interface Specification (For AI)**

This defines how the LLM interacts with your data. The Rust backend handles the logic (filtering, sorting) so the LLM doesn't have to read every file.

**4.1 MCP Tools (Capabilities)**

  * `create_task(title, context?, due_date?, priority?, notes?)`: Creates a new file with frontmatter.
  * `update_task(id, field, value)`: Patches specific frontmatter fields or appends to notes.
  * `list_tasks(status?, tag?, limit?)`: **Critical for filtering.** The AI requests "Show me active work tasks," and the tool returns a JSON list of matching headers.
  * `read_task_details(id)`: Returns the full content (frontmatter + markdown body) of a specific task.
  * `complete_task(id)`: Sets status to "done" and moves file to archive folder (optional).

**4.2 MCP Resources (Context)**

  * `daily_summary`: A dynamic resource that provides a read-only snapshot of today's high-priority tasks. This can be injected into the AI's context window automatically.

#### **5. Synchronization Strategy: "Braindead" Auto-Sync**

To minimize merge conflicts without complex locking servers.

  * **Trigger:** Every "Write" operation (Create, Update, Delete, Complete).
  * **The Flow:**
    1.  **Pre-Write:** System executes `git pull --rebase --autostash`.
    2.  **Write:** System modifies the local file.
    3.  **Post-Write:** System executes `git add . && git commit -m "Update: {task_id}" && git push`.
  * **Handling Failures:** If the push fails (internet down), the tool saves locally and flags a "Sync Pending" warning in the TUI. It retries on the next command.

#### **6. Tech Stack Recommendations**

  * **Language:** **Rust**
      * *Reasoning:* Single binary portability, strict type safety for file parsing, and instant startup time.
  * **Core Libraries:**
      * **TUI:** `ratatui` (The standard for Rust TUIs).
      * **CLI Args:** `clap` (For switching between TUI and Server modes).
      * **Serialization:** `serde` & `serde_yaml` (For parsing Frontmatter).
      * **MCP Integration:** `mcp-sdk-rs` (or a custom implementation of the JSON-RPC 2.0 protocol over stdio—it's lightweight).
      * **Git Operations:** `git2` (Rust bindings for libgit2) OR simply `std::process::Command` to call the system git binary (easier to debug).

#### **7. Functional Scope**

| Feature | TUI (Human) | MCP (AI) |
| :--- | :--- | :--- |
| **Add Task** | Form-based input or one-line command. | Tool call: `create_task` (AI parses natural language first). |
| **View Tasks** | Kanban board or List view. Sortable by date/priority. | Tool call: `list_tasks` with JSON filters. |
| **Edit Task** | Vim-like keybindings to edit text. | Tool call: `update_task` or `append_note`. |
| **Filtering** | Keybinds to filter by context (e.g., Press 'w' for Work). | Structured queries (e.g., `status="active", tag="work"`). |
| **Sync** | Auto-triggers on save. Manual sync button available. | Auto-triggers on tool execution. |

