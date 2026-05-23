---
title: Compare Command Blocks
description: Diff recent shell command inputs, outputs, and full blocks.
---

`sivtr diff` compares two recent command blocks from the current shell session. It is useful when you are iterating on builds, tests, prompts, or diagnostics and want to see what changed between runs.

Shell integration must be installed first so `sivtr` has structured command blocks to compare.

```bash
sivtr init powershell
# or: sivtr init bash / zsh / nushell
```

Restart the shell after installation.

## Basic diff

Compare the latest command output with the previous command output:

```bash
sivtr diff 1 2
```

Selectors are newest-first. `1` is the latest command block and `2` is the one before it.

## Choose what to compare

The default is output-only comparison.

```bash
sivtr diff 1 2 --output
sivtr diff 1 2 --block
sivtr diff 1 2 --input
sivtr diff 1 2 --cmd
```

| Option | Compares |
| --- | --- |
| `--output` | Command output. This is the default. |
| `--block` | Input plus output |
| `--input` | Input with prompt |
| `--cmd` | Bare command text |

Only one content mode can be used at a time.

## Side-by-side view

Use a two-column text view instead of unified diff output:

```bash
sivtr diff 2 1 --side-by-side
sivtr diff 3 1 --block --side-by-side
```

## Tips

- Compare `1 2` when you just reran a command and want to inspect the delta.
- Use `--cmd` to verify that two copied or retried commands were actually different.
- Use `--block` when prompt or command context matters as much as output.
