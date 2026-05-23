---
title: Release Notes
description: User-facing release notes for sivtr.
---

`sivtr` is in early `0.1.x` development. The CLI and configuration format may still change during this series. This page summarizes user-facing changes; the repository `CHANGELOG.md` remains the detailed changelog source.

## 0.1.3 - 2026-05-20

### Added

- Added the workspace picker experience for browsing agent sessions with richer content rendering, search navigation, scrolling, and line-numbered content views.
- Added workspace copy shortcuts for agent sessions: `i` copies user input, `o` copies assistant output, and `y` copies the whole dialogue block without role headings.
- Added project roadmap pages to the documentation site.

### Fixed

- Hardened VS Code picker command quoting across PowerShell, cmd.exe, fish, and POSIX shells.
- Ignored Claude `ai-title` metadata events instead of failing session parsing.
- Fixed CI clippy warnings.

## 0.1.2 - 2026-05-02

### Fixed

- Treat cancelling interactive pickers as a normal exit.

## 0.1.1 - 2026-05-01

### Fixed

- Fixed Codex copy picker TUI selection logic.
- Fixed terminal exit handling that could leave the terminal stuck.

## 0.1.0 - 2026-04-28

### Added

- Added `sivtr`, an early agent memory workspace for capturing command output and agent coding sessions.
- Added pipe mode with `command | sivtr`.
- Added run mode with `sivtr run <command>`.
- Added Vim-style navigation, modal interaction, visual selection, search, and clipboard copy.
- Added local SQLite history with full-text search.
- Added Codex session capture helpers with `sivtr copy codex` for reusing assistant replies, user prompts, and tool output.
- Added command-block copy, diff, and picker workflows.
- Added TOML configuration support.
- Added Windows global hotkey support for the Codex picker workflow.

## Current documented surface

The current docs cover:

- terminal pipe and run capture;
- shell session logging;
- TUI browsing and selection;
- command-block copy and diff;
- agent session copy and picker workflows for Codex, Claude Code, OpenCode, and Pi;
- workspace search and show refs;
- SQLite terminal history;
- TOML configuration;
- Windows hotkey, VS Code, tmux, Linux shortcut, and macOS launcher flows.
