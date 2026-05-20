# Changelog

All notable user-facing changes to this project are documented here.

## [0.1.3] - 2026-05-20

### Added

- Added the workspace picker experience for browsing AI sessions with richer content rendering, search navigation, scrolling, and line-numbered content views.
- Added workspace copy shortcuts for AI sessions: `i` copies user input, `o` copies assistant output, and `y` copies the whole dialogue block without role headings.
- Added project roadmap pages to the documentation site.

### Fixed

- Hardened VS Code picker command quoting across PowerShell, cmd.exe, fish, and POSIX shells.
- Ignored Claude `ai-title` metadata events instead of failing session parsing.
- Fixed CI clippy warnings.

## [0.1.2] - 2026-05-02

### Fixed

- Treat cancelling interactive pickers as a normal exit.

## [0.1.1] - 2026-05-01

### Fixed

- Fixed Codex copy picker TUI selection logic.
- Fixed terminal exit handling that could leave the terminal stuck.

## [0.1.0] - 2026-04-28

### Added

- Added `sivtr`, a terminal output workspace for capturing command output and AI coding sessions.
- Added pipe mode with `command | sivtr`.
- Added run mode with `sivtr run <command>`.
- Added Vim-style navigation, modal interaction, visual selection, search, and clipboard copy.
- Added local SQLite history with full-text search.
- Added Codex session capture helpers with `sivtr copy codex` for reusing assistant replies, user prompts, and tool output.
- Added command-block copy, diff, and picker workflows.
- Added TOML configuration support.
- Added Windows global hotkey support for the Codex picker workflow.

### Notes

- This is the first public release. The CLI and configuration format may still change during the `0.1.x` series.
