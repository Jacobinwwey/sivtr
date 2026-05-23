---
title: 架构
description: sivtr memory workspace 如何拆分为 CLI、TUI、命令处理器和 core 模块。
---

`sivtr` 是一个 Cargo workspace，主要分为两层：

- `sivtr`：位于 `src/` 的二进制 crate；
- `sivtr-core`：位于 `crates/sivtr-core/` 的库 crate。

二进制层负责用户交互：CLI 解析、命令分发、TUI 状态、workspace picker，以及平台相关的 launcher/hotkey 行为。Core crate 负责可复用的 memory 逻辑：capture、解析、buffer、selection、search primitives、history、export、config 和 Agent provider session 解析。

## Workspace 布局

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

| 区域 | 责任 |
| --- | --- |
| `cli.rs` | clap 命令定义和 help text |
| `commands/` | run、pipe、copy、history、config、hotkey、diff、import、search、show、clear 和 Codex export 处理器 |
| `commands/copy/` | 命令块复制、provider copy、workspace picker 集成和 Vim-style picker view |
| `app.rs` | 捕获输出 browser 状态机 |
| `tui/` | 终端初始化、事件处理、browser rendering、workspace rendering 和 workspace search UI |
| `command_blocks.rs` | session 浏览和复制用的命令块 span 解析 |

这一层可以依赖终端 UI 库、平台 API 和进程启动行为。

## Core crate

| 模块 | 责任 |
| --- | --- |
| `ai` | provider-neutral session、block、metadata 和 parser helpers |
| `codex`、`claude`、`opencode`、`pi` | provider-specific session 发现和解析 |
| `capture` | stdin、subprocess 和 scrollback/session capture helper |
| `parse` | ANSI stripping、Unicode display width 和 line parsing |
| `buffer` | line、cursor 和 viewport 模型 |
| `selection` | visual、line 和 block selection 提取 |
| `search` | 文本匹配和导航状态 |
| `history` | SQLite 存储、schema 和搜索 |
| `export` | clipboard、file 和 editor export helper |
| `config` | TOML 配置模型、默认值和路径解析 |
| `session` | 结构化 shell session entry 和 rendering |

这种拆分让计算和数据处理可以独立于终端 UI 测试。

## Capture flow

Pipe mode：

```text
stdin -> capture::pipe -> parse::parse_lines -> Buffer -> App -> TUI/editor
```

Run mode：

```text
subprocess -> combined output -> parse::parse_lines -> Buffer -> App -> TUI/editor
```

Session import：

```text
session log -> render entries -> parse::parse_lines -> Buffer -> command block spans -> TUI/editor
```

Command-block copy：

```text
session log -> SessionEntry list -> command blocks -> selector -> filters -> clipboard
```

Agent-provider copy：

```text
provider transcript/db -> AgentSession -> AgentBlock list -> selector -> filters -> clipboard
```

Workspace picker/search：

```text
terminal context + provider sessions -> WorkspaceSession list -> search/pick/show -> clipboard/stdout/json
```

## Provider 边界

Agent 支持在命令和 workspace 层是 provider-neutral 的。Provider 模块负责找到本地记录，并把各自事件格式转换成共享 memory block：

```text
AgentProvider -> AgentSessionProvider -> AgentSession -> AgentBlock
```

共享 workspace 代码随后可以 copy、pick、search 和 show memory，而不依赖某一个 vendor transcript 形状。

## 设计边界

Frontend 层负责呈现和交互。Rust core 做持久的 memory 工作：解析、捕获、selection 提取、搜索、存储、provider 解析和格式化。这样 UI 变化不会泄漏到 provider parser，provider 变化也不需要重写整个 CLI 表面。
