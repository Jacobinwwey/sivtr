---
title: Release Notes
description: sivtr 面向用户的发布说明。
---

`sivtr` 仍处在早期 `0.1.x` 开发阶段。本系列中 CLI 和配置格式仍可能变化。本页总结用户可见变更；仓库中的 `CHANGELOG.md` 仍是更详细的 changelog 来源。

## 0.1.3 - 2026-05-20

### Added

- 新增用于浏览 AI session 的 workspace picker 体验，包括更丰富的内容渲染、搜索导航、滚动和带行号的内容视图。
- 新增 AI session workspace copy 快捷键：`i` 复制用户输入，`o` 复制助手输出，`y` 复制不带 role heading 的完整 dialogue block。
- 新增项目 roadmap 文档页面。

### Fixed

- 加固 VS Code picker command 在 PowerShell、cmd.exe、fish 和 POSIX shells 中的 quoting。
- 忽略 Claude `ai-title` metadata event，避免 session parsing 失败。
- 修复 CI clippy warnings。

## 0.1.2 - 2026-05-02

### Fixed

- 将取消交互式 picker 视为正常退出。

## 0.1.1 - 2026-05-01

### Fixed

- 修复 Codex copy picker TUI 选择逻辑。
- 修复 terminal exit handling，避免终端卡住。

## 0.1.0 - 2026-04-28

### Added

- 新增 `sivtr`：用于捕获命令输出和 AI coding session 的终端输出 workspace。
- 新增 pipe mode：`command | sivtr`。
- 新增 run mode：`sivtr run <command>`。
- 新增 Vim 风格导航、modal interaction、visual selection、搜索和剪贴板复制。
- 新增带 full-text search 的本地 SQLite history。
- 新增 Codex session capture helper：通过 `sivtr copy codex` 复用 assistant reply、user prompt 和 tool output。
- 新增命令块 copy、diff 和 picker 工作流。
- 新增 TOML 配置支持。
- 新增 Windows 全局热键支持，用于 Codex picker 工作流。

### Notes

- 这是第一个公开版本。CLI 和配置格式在 `0.1.x` 系列中仍可能变化。

## 当前文档覆盖范围

当前文档覆盖：

- 终端 pipe 和 run capture；
- shell session logging；
- TUI 浏览和选择；
- 命令块 copy 和 diff；
- Codex、Claude Code、OpenCode 和 Pi 的 AI session copy 与 picker 工作流；
- workspace search 和 show refs；
- SQLite terminal history；
- TOML 配置；
- Windows hotkey、VS Code、tmux、Linux shortcut 和 macOS launcher 流程。
