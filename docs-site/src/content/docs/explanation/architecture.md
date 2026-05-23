---
title: Architecture
description: How the sivtr memory workspace is split between CLI, TUI, command handlers, and core modules.
---

`sivtr` is a Cargo workspace with two main layers:

- `sivtr`, the binary crate in `src/`;
- `sivtr-core`, the library crate in `crates/sivtr-core/`.

The binary owns user interaction: CLI parsing, command dispatch, TUI state, workspace pickers, and platform-specific launcher/hotkey behavior. The core crate owns reusable memory logic: capture, parsing, buffers, selection, search primitives, history, export, config, and agent-provider session parsing.

## Workspace layout

```text
sivtr/
|- Cargo.toml
|- src/
|  |- cli.rs
|  |- main.rs
|  |- app.rs
|  |- commands/
|  `- tui/
`- crates/
   `- sivtr-core/
      `- src/
         |- ai.rs
         |- buffer/
         |- capture/
         |- claude.rs
         |- codex.rs
         |- config/
         |- export/
         |- history/
         |- opencode.rs
         |- parse/
         |- pi.rs
         |- search/
         |- selection/
         `- session/
```

## Binary crate

| Area | Responsibility |
| --- | --- |
| `cli.rs` | clap command definitions and help text |
| `commands/` | handlers for run, pipe, copy, history, config, hotkey, diff, import, search, show, clear, and Codex export |
| `commands/copy/` | command-block copy, provider copy, workspace picker integration, and Vim-style picker view |
| `app.rs` | captured-output browser state machine |
| `tui/` | terminal setup, event handling, browser rendering, workspace rendering, and workspace search UI |
| `command_blocks.rs` | parsed command-block spans for session browsing and copying |

This layer can depend on terminal UI libraries, platform APIs, and process spawning behavior.

## Core crate

| Module | Responsibility |
| --- | --- |
| `ai` | provider-neutral session, block, metadata, and parser helpers |
| `codex`, `claude`, `opencode`, `pi` | provider-specific session discovery and parsing |
| `capture` | stdin, subprocess, and scrollback/session capture helpers |
| `parse` | ANSI stripping, Unicode display width, and line parsing |
| `buffer` | line, cursor, and viewport models |
| `selection` | visual, line, and block selection extraction |
| `search` | text matching and navigation state |
| `history` | SQLite storage, schema, and search |
| `export` | clipboard, file, and editor export helpers |
| `config` | TOML config model, defaults, and path resolution |
| `session` | structured shell session entries and rendering |

This split keeps computation and data handling testable independently from the terminal UI.

## Capture flow

Pipe mode:

```text
stdin -> capture::pipe -> parse::parse_lines -> Buffer -> App -> TUI/editor
```

Run mode:

```text
subprocess -> combined output -> parse::parse_lines -> Buffer -> App -> TUI/editor
```

Session import:

```text
session log -> render entries -> parse::parse_lines -> Buffer -> command block spans -> TUI/editor
```

Command-block copy:

```text
session log -> SessionEntry list -> command blocks -> selector -> filters -> clipboard
```

Agent-provider copy:

```text
provider transcript/db -> AgentSession -> AgentBlock list -> selector -> filters -> clipboard
```

Workspace picker/search:

```text
terminal context + provider sessions -> WorkspaceSession list -> search/pick/show -> clipboard/stdout/json
```

## Provider boundary

Agent support is provider-neutral at the command and workspace layers. Provider modules are responsible for finding local records and converting provider-specific event formats into shared memory blocks:

```text
AgentProvider -> AgentSessionProvider -> AgentSession -> AgentBlock
```

The shared workspace code can then copy, pick, search, and show memory without depending on one vendor transcript shape.

## Design boundary

The frontend layer is presentation and interaction. The Rust core performs the durable memory work: parsing, capture, selection extraction, search, storage, provider parsing, and formatting. This keeps UI changes from leaking into provider parsers and keeps provider changes from rewriting the whole CLI surface.
