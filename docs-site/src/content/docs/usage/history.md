---
title: History
description: List, search, show, and retain saved terminal captures.
---

`sivtr` stores captured terminal output in a local SQLite history database with FTS5 search. History is for longer-lived terminal captures; it is separate from per-shell session logs and agent transcripts.

## List recent entries

```bash
sivtr history
sivtr history list
sivtr history list --limit 50
```

Output includes the entry id, timestamp, command, and a preview of the content.

## Search history

```bash
sivtr history search "panic"
sivtr history search "failed assertion" --limit 10
```

Search uses the history full-text index. Use the resulting id with `history show`.

This is different from `sivtr search`, which searches current-workspace agent sessions plus the current terminal session log when shell integration has data.

## Show an entry

```bash
sivtr history show 42
```

The detail view prints metadata followed by the stored content:

- id;
- timestamp;
- command;
- source;
- host;
- content.

## Retention

History retention is controlled by config:

```toml
[history]
auto_save = true
max_entries = 0
```

`max_entries = 0` means unlimited.

Disable automatic history saves:

```toml
[history]
auto_save = false
```

## When to use history vs session logs

| Need | Use |
| --- | --- |
| Search older captured terminal output | `sivtr history search` |
| Copy the latest shell command output | `sivtr copy out` |
| Browse the current shell's structured blocks | `sivtr import` |
| Search current workspace terminal and agent memory | `sivtr search` |
