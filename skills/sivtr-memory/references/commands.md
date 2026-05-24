# Command Cookbook

Use these commands as starting points. This file is the single source for `sivtr` command syntax.
Prefer small, targeted queries over dumping large histories.

## Search Commands

### Search option reference

`sivtr search` takes a regex query and composable filters. Prefer filters for
metadata and keep the query focused on the thing to find.

```bash
sivtr search "<query>" \
  --shell|--agent \
  --provider <all|codex|claude|pi|opencode> \
  --scope <content|dialogue|session> \
  --cwd <path> \
  --recent <count|duration> \
  --since <time> \
  --until <time> \
  --json \
  --limit <n>
```

- `--shell`: search terminal command records only. Use this for "latest terminal error" / "刚才终端报错".
- `--agent`: search AI/agent conversation records only. Use with `--provider` when the user names a provider, such as "pi 中的", "codex 里", or "Claude 上次".
- `--scope content`: search matched text/content; this is the default.
- `--scope dialogue`: search dialogue or command block titles.
- `--scope session`: search session titles.
- `--cwd`: choose the workspace used to resolve current AI sessions. Omit it
  when already running in the target repo.
- `--recent`: search only recent content. Accepts counts (`20`) or durations
  (`30m`, `2h`, `7d`).
- `--since` / `--until`: bound search by RFC3339 time, Unix seconds/millis, or
  relative durations.
- `--limit`: cap results. Start at `20`; use `30` for handoff/recap.
- `--json`: required for agent use unless the user explicitly wants raw text.

### General search

```bash
sivtr search "<case-insensitive-regex>" --json --limit 20
```

### Search latest terminal errors

For "最新终端报错" / "刚才终端报错", search shell records directly, then expand the newest shell ref.

```bash
sivtr search "Error|error|failed|fatal|panic|Traceback|Exception|exit code|not found|External command failed|No such file or directory|permission denied|is not recognized" --shell --json --limit 20
```

If the returned ref ends with a line number, remove the trailing line segment and run `sivtr show "<block-ref>" --json` before answering.

### Search terminal + AI memory for common errors

```bash
sivtr search "error|failed|panic|Traceback|Exception|exit code|could not compile|FAILED" --json --limit 20
```

### Rust failures

```bash
sivtr search "error\\[E[0-9]+\\]|panicked|test result: FAILED|could not compile|borrow|lifetime" --json --limit 20
```

### JavaScript / TypeScript failures

```bash
sivtr search "TypeError|ReferenceError|TS[0-9]+|npm ERR|pnpm|vite|webpack|ELIFECYCLE|failed" --json --limit 20
```

### Python failures

```bash
sivtr search "Traceback|ModuleNotFoundError|ImportError|AssertionError|pytest|FAILED|Exception" --json --limit 20
```

### Previous decisions or AI discussion

```bash
sivtr search "lazy load|workspace TUI|metadata scan|decision|TODO|next step" --agent --json --limit 20
```

### Search titles instead of content

```bash
sivtr search "workspace picker" --scope session --json --limit 20
sivtr search "cargo test" --scope dialogue --json --limit 20
```

### Provider-specific search

```bash
sivtr search "<query>" --agent --provider codex --json --limit 20
sivtr search "<query>" --agent --provider claude --json --limit 20
sivtr search "<query>" --agent --provider pi --json --limit 20
sivtr search "<query>" --agent --provider opencode --json --limit 20
```

### Compose filters from the request

Map request constraints to options instead of hard-coding scenario-specific
queries. Keep the regex query for the content/topic being searched.

```bash
sivtr search "<topic>" --agent --provider <provider> --recent <duration> --json --limit 20
sivtr search "<topic>|<related-term>|<status-term>" --agent --provider <provider> --recent <duration> --json --limit 30
sivtr search "<topic>" --scope <content|dialogue|session> --cwd <path> --json --limit 20
```

Examples of the mapping:

- "pi 中的 merge" -> query `merge`, options `--agent --provider pi`
- "最近两小时的终端报错" -> error query, options `--shell --recent 2h`
- "这个仓库上次的 CI 失败" -> CI/failure query, option `--cwd <repo>`
- "标题里有 workspace picker" -> query `workspace picker`, option `--scope session` or `--scope dialogue`

## JSON Handling

Treat `--json` output as structured evidence, not as a free-form transcript.

`sivtr search --json` returns a wrapper with `query`, `scope`, `cwd`,
`match_count`, and `results`. Inspect these result fields first:

- `ref`: stable reference for follow-up expansion
- `kind`: `shell` or `ai`
- `timestamp`: how recent it is
- `title.session`: session title
- `title.dialogue`: dialogue or command block title, when available
- `content`: matched line or extracted content

Expected result item shape:

```json
{
  "ref": "terminal/current/12/8",
  "kind": "shell",
  "timestamp": "...",
  "title": {
    "session": "current shell",
    "dialogue": "cargo test"
  },
  "content": "test result: FAILED"
}
```

Use `ref` for precise follow-up. Do not infer provider/session identity from
display text when a `ref` is available.

## Expansion Commands

Use expansion after search identifies a target. Prefer small, precise expansions.

### Show a matched ref

Use `show` when search returned a `ref` and you need exact content.

```bash
sivtr show "<ref>" --json
sivtr show "terminal/current/12/8" --json
```

Refs have this shape:

```text
source/session[/dialogue[/line]]
```

When a search result points to a single line but the surrounding block matters,
show the block ref by removing the trailing line segment. For example, expand
`terminal/session_8620/8/3` with:

```bash
sivtr show "terminal/session_8620/8" --json
```

Avoid clipboard-oriented `copy` workflows in agent skills; they are primarily
for human selection and clipboard use.

## Query Construction Tips

- Use the exact tool name when known: `cargo test`, `pytest`, `npm ERR`, `wrangler deploy`.
- Include high-signal error tokens: `panic`, `Traceback`, `TS2307`, `error[E`, `exit code`.
- Search for decision words when reconstructing context: `decision`, `defer`, `blocked`, `next step`, `TODO`.
- Start with `--limit 20`; increase only if the result set is clearly incomplete.

## Token Budget

- Start with `--limit 20` for normal searches.
- Use `--limit 30` only for handoff or recap work.
- Narrow the query before increasing the limit.
- Prefer `sivtr show "<ref>" --json` when search returns a useful ref.
- If the shown line is too narrow, remove the trailing line segment from the ref and show the whole block.
