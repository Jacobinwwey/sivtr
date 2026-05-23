---
title: Keybindings
description: Browser, command-block, picker, search, line-filter, and Vim-view keybindings.
---

## Browser

The browser is used by pipe mode, run mode, and session import.

| Key | Mode | Action |
| --- | --- | --- |
| `j` / `Down` | Normal, Insert, Visual | Move down |
| `k` / `Up` | Normal, Insert, Visual | Move up |
| `h` / `Left` | Normal, Insert, Visual | Move left |
| `l` / `Right` | Normal, Insert, Visual | Move right |
| `0` / `Home` | Normal, Insert, Visual | Start of line |
| `^` | Normal, Insert, Visual | First non-blank |
| `$` / `End` | Normal, Insert, Visual | End of line |
| `Ctrl-D` | Normal, Insert, Visual | Half page down |
| `Ctrl-U` | Normal, Insert, Visual | Half page up |
| `Ctrl-F` / `PageDown` | Normal, Insert, Visual | Page down |
| `Ctrl-B` / `PageUp` | Normal, Insert, Visual | Page up |
| `gg` | Normal, Visual | Top |
| `G` | Normal, Visual | Bottom |
| `H` | Normal, Visual | Top of viewport |
| `M` | Normal, Visual | Middle of viewport |
| `L` | Normal, Visual | Bottom of viewport |
| `/` | Normal, Insert | Search |
| `n` | Normal | Next match |
| `N` | Normal | Previous match |
| `i` | Normal | Insert mode |
| `v` | Normal, Visual | Character selection |
| `V` | Normal, Visual | Line selection |
| `Ctrl-V` | Normal, Visual | Block selection |
| `o` | Visual | Swap selection anchor |
| `y` | Visual | Copy selection |
| `e` | Normal, Visual | Open editor |
| `Esc` | Insert, Visual, Search | Cancel current mode |
| `q` | Normal | Quit |

## Command-block navigation

Only available when the buffer has parsed command blocks.

| Key | Action |
| --- | --- |
| `[[` | Previous command block |
| `]]` | Next command block |
| `myy` | Copy current command block |
| `myi` | Copy current command input |
| `myo` | Copy current command output |
| `myc` | Copy current bare command |
| `mvv` | Select current command block |
| `mvi` | Select current command input |
| `mvo` | Select current command output |

## Workspace picker

The workspace picker is used by agent-session pickers and command-block pickers.

| Key | Action |
| --- | --- |
| `0` | Focus Source pane |
| `1` | Focus Sessions pane |
| `2` | Focus Dialogues pane |
| `3` | Focus Content pane |
| `j` / `Down` | Move down in focused pane |
| `k` / `Up` | Move up in focused pane |
| `h` / `Left` | Focus previous pane |
| `l` / `Right` | Focus next pane |
| `Space` | Toggle current source, session, or dialogue |
| `a` | Select all sources or toggle all dialogues, depending on pane |
| `g` | Select agent sources in Source pane |
| `t` | Select terminal source in Source pane, or open Vim-style full view elsewhere |
| `v` | Range-select dialogues, or start visual text selection in Content pane |
| `:` | Start a temporary line filter for the next copy |
| `i` | Copy input/question |
| `o` | Copy output/answer |
| `y` | Copy input + output block |
| `c` | Copy bare command when available |
| `Enter` | Enter pane or copy current selection |
| `Ctrl-D` / `PageDown` | Scroll Content down |
| `Ctrl-U` / `PageUp` | Scroll Content up |
| `r` | Toggle raw/read content mode in Content pane |
| `z` | Toggle focused pane fullscreen |
| `/` | Search all sessions |
| `?` | Open help |
| `q` / `Esc` | Cancel or go back |

## Workspace search

| Key | Action |
| --- | --- |
| `/` | Open search input |
| `Enter` | Accept search input |
| `Esc` | Clear or close search |
| `Backspace` | Edit query while input is open |
| `n` | Next match |
| `N` | Previous match |

Search prefixes:

| Prefix | Scope |
| --- | --- |
| none | Content |
| `#` | Dialogue titles |
| `>` | Session titles |

## Line filter input

Opened from the workspace picker with `:`.

| Key | Action |
| --- | --- |
| digits, `,`, `:` | Build a 1-based line spec |
| `Backspace` | Edit pending filter |
| `Enter` | Apply to the next copy |
| `Esc` | Clear/cancel |

Examples: `2:8`, `1,3,8:12`.

## Vim-style full view

This view is opened from the picker with `t`.

| Key | Action |
| --- | --- |
| `[[` | Previous block |
| `]]` | Next block |
| `myy` | Copy block |
| `myi` | Copy input |
| `myo` | Copy output |
| `myc` | Copy command |
| `mvv` | Select block |
| `mvi` | Select input |
| `mvo` | Select output |
| `T` | Toggle alternate tool view when available |
| `p`, `q`, `Esc` | Return to picker |
