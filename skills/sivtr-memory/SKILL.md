---
name: sivtr-memory
description: "Retrieve shared local work memory: terminal activity, AI conversation history, prior decisions, validation evidence, debugging trails, recaps, and handoff context. Use before asking the user to repeat local work context."
---

# Sivtr Memory

Sivtr is the shared local work memory for this machine.

Use it before asking the user to paste logs, repeat decisions, or restate earlier work.

Core rule:

> Search for evidence first. Expand only the smallest relevant context. Ask when memory is missing, ambiguous, stale, or permission is required.

## When to Use

Use this skill whenever local work memory may contain useful evidence. Sivtr retrieves recent terminal activity, AI conversation history, prior decisions, validation output, debugging trails, and handoff context.

Use it before asking the user to repeat context when you need to know things like:

- what just happened in the terminal or an agent session
- what command was run, what output it produced, or whether validation passed
- what the user or a previous agent decided, rejected, or planned
- where earlier debugging, build, test, lint, deploy, or research work left off
- what evidence supports a recap, handoff, status update, or next step
- whether missing or truncated current-context output exists in local memory

Do not use Sivtr as truth by itself. Treat memory as evidence to retrieve, then verify current files or commands before making claims about current state.

## Default Retrieval Workflow

1. Convert the user's vague reference into a small query.
2. Choose a source: `terminal`, `agent`, `pi`, `codex`, `claude`, `opencode`, a WorkRef selector, or a WorkSet reference such as `@last` / `@name[1,3]`.
3. Search with a small limit. Put content terms in `-m` / `--match`. Use `--last` / `--since` for time windows and `-i` / `--in` for field constraints.
   - Latest terminal error: `sivtr s terminal --status fail --latest 1 --refs`
   - Broader terminal error scan: `sivtr s terminal -m "Error|error|failed|fatal|not found|External command failed" --latest 20 --refs`
4. Refine the current WorkSet when needed:
   - `sivtr s @last -m "more specific terms" --refs`
   - `sivtr s @last[1,3] -m "narrower terms" --refs`
5. Expand only useful records:
   - `sivtr zoom @last[1] -C 2 --save ctx --refs`
   - `sivtr show @ctx --full`
   - `sivtr show <ref> --full`
6. Answer with evidence, then verify current files or commands when the claim depends on present repository state.
7. Ask the user only after local memory has been checked and still lacks the needed fact.

## WorkSet Flow

`search` and `zoom` create WorkSets. Each run saves the result to `@last`. Add `--save <name>` to keep a named WorkSet.

Source forms:

- `terminal`, `agent`, `pi`, `codex`, `claude`, `opencode`
- `terminal/<session>/<record>`, `<provider>/<session>/<turn>`, and selector variants
- `@last`, `@name`, `@name[1]`, `@name[1,3]`, `@name[1..5]`, `@name[1..3,8]`
- `@` reads a WorkSet from stdin in shell pipelines

Output behavior:

- Terminal stdout with no explicit format prints `full`.
- Piped stdout with no explicit format prints WorkSet JSON for the next command.
- `--json` is `--format workset`.
- `--refs` is `--format refs`.
- `--full` on `show` is `--format full`.

Pipeline example:

```bash
sivtr s agent -m "panic|failed" --latest 20 \
  | sivtr s @ -m "cargo|test" \
  | sivtr zoom @ -C 1 \
  | sivtr show @ -f timeline
```

Named WorkSet example:

```bash
sivtr s agent -m "decision|TODO|next step" --latest 20 --save hits --refs
sivtr s @hits -m "workset|zoom|show" --save narrowed --refs
sivtr zoom @narrowed[1] -C 2 --save ctx --refs
sivtr show @ctx --full
```

## Non-Interactive Safety Rules

- Prefer non-interactive commands: `sivtr s ... --refs`, `sivtr s ... -f <timeline|compact|md>`, `sivtr show ... --full`, `sivtr show ... --json`.
- Use WorkSet refs and `@` pipelines for chaining.
- Do not open TUI pickers (`--pick`, hotkey picker) unless the user explicitly wants interactive selection.
- Do not run `sivtr clear`, hotkey start/stop, shell init, or config mutation unless the user explicitly asks.
- Avoid clipboard-oriented workflows in agent retrieval. Use refs from `search` and expand them with `show` / `zoom`.
- Avoid dumping huge histories into the model. Search narrowly first, then expand only the relevant records.
- If `sivtr` is not installed or no session log exists, say so briefly and continue with normal tools. Do not invent memory results.

## Load References as Needed

References are relative to this skill directory.

- `references/commands.md` — command syntax, WorkSet handling, and token budget.
- `references/patterns.md` — common user intents mapped to retrieval steps.
- `references/evidence.md` — what counts as evidence and how to report it.

Read only the file needed for the current task.
