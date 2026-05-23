---
title: CLI Reference
description: Command syntax, subcommands, options, providers, selectors, and examples.
---

This page documents the public CLI surface. The source of truth is `src/cli.rs`; run `sivtr --help` and `sivtr <command> --help` for installed-version help.

## Top-level

```bash
sivtr [COMMAND]
```

If no command is provided, `sivtr` reads from stdin, matching pipe mode.

## run

```bash
sivtr run <COMMAND> [ARGS...]
```

Runs a command, captures combined stdout/stderr, reports the exit status, saves history when enabled, and opens the captured output.

```bash
sivtr run cargo test
sivtr run git status --short
```

## pipe

```bash
sivtr pipe
```

Reads stdin and opens it. Piping directly to `sivtr` is equivalent:

```bash
cargo build 2>&1 | sivtr
```

## import

```bash
sivtr import
```

Opens the current structured shell session log. Requires shell integration.

## init

```bash
sivtr init <TARGET>
```

Supported targets:

| Target | Purpose |
| --- | --- |
| `powershell` | Install Windows PowerShell hook |
| `pwsh` | Alias for PowerShell integration |
| `bash` | Install Bash hook |
| `zsh` | Install Zsh hook |
| `nushell` / `nu` | Install Nushell hook |
| `tmux` | Install tmux picker binding |
| `linux-shortcut` | Generate Linux desktop/terminal picker launcher |
| `macos-shortcut` | Generate macOS Terminal/LaunchAgent picker launcher |

## copy

```bash
sivtr copy [MODE] [SELECTOR] [OPTIONS]
```

Command-block modes:

| Mode | Meaning |
| --- | --- |
| no mode | Copy input plus output |
| `in` | Copy input |
| `out` | Copy output |
| `cmd` | Copy bare command |

Aliases:

| Alias | Expands to |
| --- | --- |
| `sivtr c` | `sivtr copy` |
| `sivtr ci` | `sivtr copy in` |
| `sivtr co` | `sivtr copy out` |
| `sivtr cc` | `sivtr copy cmd` |

Common options:

| Option | Meaning |
| --- | --- |
| `--ansi` | Copy ANSI-decorated text when available |
| `--pick` | Open the interactive picker |
| `--print` | Print copied text after copying |
| `--regex <PATTERN>` | Keep lines matching regex |
| `--lines <SPEC>` | Keep selected 1-based lines |

Input-capable modes also support:

| Option | Meaning |
| --- | --- |
| `--prompt <TEXT>` | Rewrite the copied input prompt |

Examples:

```bash
sivtr copy
sivtr copy 3 --print
sivtr copy --prompt ":"
sivtr copy in 2..4
sivtr copy out --pick --regex panic
sivtr copy cmd --pick
```

## copy agent provider sessions

```bash
sivtr copy <PROVIDER> [MODE] [SELECTOR] [OPTIONS]
```

Providers:

| Provider | Command |
| --- | --- |
| Codex | `sivtr copy codex` |
| Claude Code | `sivtr copy claude` |
| OpenCode | `sivtr copy opencode` |
| Pi | `sivtr copy pi` |

Modes:

| Mode | Meaning |
| --- | --- |
| no mode | Last completed user + assistant turn |
| `in` | Last user message |
| `out` | Last assistant reply |
| `tool` | Last tool output |
| `all` | Whole parsed session |

Agent copy options include all common copy options plus:

| Option | Meaning |
| --- | --- |
| `--session <N|ID>` | Select the Nth newest selectable session, or match an id/id prefix |

Examples:

```bash
sivtr copy claude
sivtr copy claude out --print
sivtr copy claude --session 2
sivtr copy codex 2..4
sivtr copy codex out --pick
sivtr copy opencode all --lines 1:20
sivtr copy pi tool --regex error
```

## diff

```bash
sivtr diff <LEFT> <RIGHT> [OPTIONS]
```

Compares two recent command blocks from the current shell session. Each selector must resolve to exactly one block.

Content options:

| Option | Meaning |
| --- | --- |
| `--output` | Compare output text. This is the default. |
| `--block` | Compare input plus output |
| `--input` | Compare input with prompt |
| `--cmd` | Compare bare command text |

View option:

| Option | Meaning |
| --- | --- |
| `--side-by-side` | Show a two-column text view |

Examples:

```bash
sivtr diff 1 2
sivtr diff 3 1 --block
sivtr diff 2 1 --side-by-side
```

## search

```bash
sivtr search <QUERY> [OPTIONS]
```

Searches current-workspace agent sessions and the current terminal session log when shell integration has data.

Options:

| Option | Meaning |
| --- | --- |
| `--scope <SCOPE>` | `content`, `dialogue`, or `session`; default is `content` |
| `--provider <PROVIDER>` | `all`, `codex`, `claude`, `opencode`, or `pi`; default is `all` |
| `--cwd <PATH>` | Workspace directory used to resolve sessions |
| `-l, --limit <N>` | Maximum results to print; default is `20` |
| `--json` | Print machine-readable JSON |

Examples:

```bash
sivtr search panic
sivtr search "workspace picker" --scope dialogue
sivtr search sivtr --scope session --provider codex
sivtr search "build error" --json --limit 20
```

## show

```bash
sivtr show <REF> [OPTIONS]
```

Prints a workspace ref from an agent provider or the current terminal session.

Ref syntax:

```text
source/session[/dialogue[/line]]
```

Options:

| Option | Meaning |
| --- | --- |
| `--cwd <PATH>` | Workspace directory used to resolve sessions |
| `--json` | Print machine-readable JSON |

Examples:

```bash
sivtr show claude/<session-id>
sivtr show claude/<session-id>/3
sivtr show claude/<session-id>/3/7 --json
sivtr show terminal/current/2
```

## history

```bash
sivtr history [COMMAND]
```

Subcommands:

| Command | Meaning |
| --- | --- |
| `list [-l, --limit <N>]` | List recent entries |
| `search <KEYWORD> [-l, --limit <N>]` | Search saved capture history |
| `show <ID>` | Show a specific history entry |

If no history subcommand is provided, `list` is used.

## config

```bash
sivtr config [COMMAND]
```

Subcommands:

| Command | Meaning |
| --- | --- |
| `show` | Show config path and content |
| `init` | Create default config |
| `edit` | Open config in editor |

If no config subcommand is provided, `show` is used.

## hotkey

```bash
sivtr hotkey [COMMAND]
```

Subcommands:

| Command | Meaning |
| --- | --- |
| `start [--chord <CHORD>] [--provider <PROVIDER>]` | Start Windows global hotkey daemon |
| `status` | Show daemon status |
| `stop` | Stop daemon |

If no hotkey subcommand is provided, `status` is used.

Examples:

```bash
sivtr hotkey start
sivtr hotkey start --chord alt+y
sivtr hotkey start --provider claude
sivtr hotkey status
sivtr hotkey stop
```

## codex export

```bash
sivtr codex export --dest <PATH> [OPTIONS]
```

Exports local Codex rollout JSONL files into a target directory containing a `sessions/` tree.

Options:

| Option | Meaning |
| --- | --- |
| `--dest <PATH>` | Destination directory that will receive the `sessions/` tree |
| `--limit <N>` | Keep only newest N session files; `0` means export all |
| `--watch` | Continue mirroring local sessions |
| `--interval <SECONDS>` | Seconds between sync passes when watching; default is `1` |
| `--interval-ms <MILLISECONDS>` | Milliseconds between sync passes; overrides `--interval` |

Examples:

```bash
sivtr codex export --dest /srv/sivtr/root-codex
sivtr codex export --dest /srv/sivtr/root-codex --watch
sivtr codex export --dest /srv/sivtr/root-codex --limit 100
```

## clear

```bash
sivtr clear [--all]
```

Clears current shell session logs. `--all` clears all recorded session logs and state files managed by `sivtr`.

## Shared syntax

See [Selectors and Filters](/reference/selectors-and-filters/) for recency selectors, `--session`, providers, `--regex`, `--lines`, `--ansi`, `--print`, and workspace refs.
