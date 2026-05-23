---
title: Search and Show Results
description: Search current workspace memory and print exact refs.
---

`sivtr search` queries current-workspace agent sessions plus the current terminal session log when shell integration has data. `sivtr show` prints the content behind an exact ref.

Use them together when an interactive picker is too much and you want scriptable memory for a human workflow, an agent prompt, or another tool. They are also the safest primitives for skills because they can run non-interactively and return exact refs.

For example, a "fix the terminal error" skill can start with:

```bash
sivtr search "error|failed|panic|Traceback|Exception|exit code|FAILED" --json --limit 20
sivtr copy out 1 --print
```

and a "recent work timeline" skill can combine search results with recent command titles:

```bash
sivtr search "TODO|next step|decision|test result|passed|failed|commit|build" --json --limit 50
sivtr copy cmd 1..20 --print
```

## Search content

```bash
sivtr search panic
sivtr search "workspace picker"
sivtr search "build error" --limit 20
```

By default, search uses the current directory as the workspace and searches dialogue/content text across all supported agent providers. When a current shell session log exists, it is included as a `terminal/current` source.

## Search scopes

```bash
sivtr search "panic" --scope content
sivtr search "release notes" --scope dialogue
sivtr search "sivtr" --scope session
```

| Scope | Searches |
| --- | --- |
| `content` | Dialogue/content lines. This is the default. |
| `dialogue` | Dialogue titles |
| `session` | Session titles |

Search queries are case-insensitive regexes, matching the workspace picker search behavior.

## Provider filters

```bash
sivtr search "panic" --provider all
sivtr search "panic" --provider codex
sivtr search "panic" --provider claude
sivtr search "panic" --provider opencode
sivtr search "panic" --provider pi
```

Use `all` to search every supported provider.

## Workspace directory

Override the workspace directory used for session resolution:

```bash
sivtr search "panic" --cwd /path/to/project
sivtr show claude/<session>/<dialogue> --cwd /path/to/project
```

This is useful from scripts or editor integrations that run outside the target project directory.

## JSON output

Use JSON when another tool needs refs and content:

```bash
sivtr search "build error" --json --limit 20
```

The JSON output includes:

- query;
- scope;
- cwd;
- total match count;
- result list with `ref`, kind, timestamp, title, and content.

## Show a ref

Refs have this shape:

```text
source/session[/dialogue[/line]]
```

Print the whole session content:

```bash
sivtr show claude/<session>
```

Print one dialogue:

```bash
sivtr show claude/<session>/<dialogue>
```

Print one 1-based line from one dialogue:

```bash
sivtr show claude/<session>/<dialogue>/<line>
```

Print terminal search results too:

```bash
sivtr show terminal/current
sivtr show terminal/current/<block>
sivtr show terminal/current/<block>/<line>
```

Use JSON for machine-readable output:

```bash
sivtr show pi/<session>/<dialogue>/<line> --json
```

## Practical loop

1. Search broadly:

   ```bash
   sivtr search "panic" --provider all --json --limit 50
   ```

2. Pick the result ref you care about.
3. Print the surrounding dialogue:

   ```bash
   sivtr show <source/session/dialogue>
   ```

4. Use the exact line ref when you need a compact citation, script input, or context handle for another agent.
