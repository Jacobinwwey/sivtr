---
title: Copy Command Blocks
description: Copy recent shell input, output, commands, ranges, and filtered lines.
---

`sivtr copy` reads the structured shell session log created by shell integration. It does not require opening the browser.

Install shell integration first if command-block copy has no data:

```bash
sivtr init powershell
# or: sivtr init bash / zsh / nushell
```

Restart the shell after installation.

## Basic modes

```bash
sivtr copy
sivtr copy in
sivtr copy out
sivtr copy cmd
```

| Command | Copies |
| --- | --- |
| `sivtr copy` | Input plus output |
| `sivtr copy in` | Input only, including prompt by default |
| `sivtr copy out` | Output only |
| `sivtr copy cmd` | Bare command only |

Aliases:

| Alias | Full command |
| --- | --- |
| `sivtr c` | `sivtr copy` |
| `sivtr ci` | `sivtr copy in` |
| `sivtr co` | `sivtr copy out` |
| `sivtr cc` | `sivtr copy cmd` |

## Select recent blocks

Selectors are relative to the newest command block:

```bash
sivtr copy 1
sivtr copy out 2
sivtr copy in 2..4
```

`1` is the latest block. `2` is the previous block. `2..4` selects several recent blocks. See [Selectors and Filters](/reference/selectors-and-filters/) for the shared syntax.

## Print after copying

Use `--print` to see what was copied:

```bash
sivtr copy out --print
```

The text is still copied to the clipboard.

## Preserve ANSI

Use `--ansi` when you want colored terminal sequences preserved:

```bash
sivtr copy out --ansi
```

This only has an effect when the session entry has ANSI-preserved output.

## Rewrite the prompt

Input-copying modes preserve the original prompt by default. Override it with `--prompt`:

```bash
sivtr copy in --prompt ":"
sivtr copy --prompt ">"
```

If the prompt does not end with whitespace, `sivtr` inserts one space before the command.

## Filter copied text

Filters run after selected blocks are assembled.

```bash
sivtr copy out --regex panic
sivtr copy out --lines 10:20
sivtr copy out --lines 1,3,8:12
```

If both filters are set, `--regex` runs first and `--lines` runs on the filtered result.

## Interactive picker

Open an interactive picker:

```bash
sivtr copy --pick
sivtr copy out --pick
sivtr copy cmd --pick
```

Useful picker keys:

| Key | Action |
| --- | --- |
| `j` / `k` | Move |
| `Space` | Toggle current entry |
| `v` | Mark range anchor |
| `a` | Toggle all |
| `:` | Start a temporary line filter for the next copy |
| `p` | Toggle preview |
| `t` | Open Vim-style full view |
| `Enter` | Confirm |
| `Backspace` | Edit the pending line filter |
| `Esc` | Cancel |

The Vim-style full view supports `[[` and `]]` to jump blocks, `myy` / `myi` / `myo` / `myc` to copy, and `mvv` / `mvi` / `mvo` to select.
