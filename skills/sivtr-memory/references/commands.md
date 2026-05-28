# Command Cookbook

Use these commands as starting points. Prefer small, targeted queries over dumping large histories.

## Core Model

`search`, `zoom`, and `show` all take a source.

Source forms:

- `terminal`: terminal command records
- `agent`: AI/agent conversation records from all providers
- `codex`, `claude`, `pi`, `opencode`: one provider's conversation records
- `terminal/<session>/<record>/<line>` or `<provider>/<session>/<turn>/<line>` with optional trailing segments
- `<provider>/<session>/<turn>/<i|o>/<part>` for input/output part refs
- `@last`, `@name`, `@name[1]`, `@name[1,3]`, `@name[1..5]`, `@name[1..3,8]`
- `@` to read a WorkSet from stdin

WorkSet selector indexes are 1-based. Discrete selectors keep the requested order.

## Search

`search` filters records from a source and creates a WorkSet. Every search saves the result to `@last`; `--save <name>` also saves it as `@name`.

```bash
sivtr s <source> \
  -m <case-insensitive-regex> \
  -i <content|title|session|input|output|command|all> \
  -v <case-insensitive-regex> \
  --status <success|failure|unknown|fail> \
  --exit-code <code> \
  --min-duration <duration> \
  --max-duration <duration> \
  --sort <newest|oldest|duration|duration-asc|exit-code|exit-code-asc> \
  --cwd <path> \
  --last <duration> \
  --since <time> \
  --until <time> \
  --latest <n> \
  --limit <n> \
  --save <name> \
  -f <full|timeline|compact|md|refs|workset>
```

Filters:

- `-m` / `--match`: case-insensitive regex content filter.
- `-v` / `--exclude`: case-insensitive regex exclusion filter on the same `-i` / `--in` search surface.
- `-i` / `--in`: field for `--match` and `--exclude`; default is `content`.
- `--status`: filter by command/record outcome.
- `--exit-code`: filter terminal records by exact process exit code.
- `--min-duration` / `--max-duration`: filter by duration (`500ms`, `2s`, `1m`, `1h`).
- `--sort`: sort results (`newest`, `oldest`, `duration`, `duration-asc`, `exit-code`, `exit-code-asc`).
- `--cwd`: choose the workspace used to resolve current AI sessions.
- `--last`: relative time window (`30m`, `2h`, `7d`).
- `--since` / `--until`: bound search by RFC3339 time, Unix seconds/millis, or relative durations.
- `--latest`: return the latest N matching records.
- `--limit`: cap printed results.
- `--save`: name the result WorkSet.

Output formats:

- `full`: complete record content.
- `timeline`: timestamp, provider/source, ref, and title.
- `compact`: concise human-readable records.
- `md`: markdown export.
- `refs`: plain refs.
- `workset`: WorkSet JSON.
- `--json`: alias for `-f workset`.
- `--refs`: alias for `-f refs`.

Default output:

- Terminal stdout: `full`.
- Piped stdout: `workset`.

## General Search

```bash
sivtr s agent -m "<case-insensitive-regex>" --latest 20 --refs
sivtr s terminal -m "<case-insensitive-regex>" --latest 20 --refs
sivtr show @last -f timeline
```

## Search Latest Terminal Errors

```bash
sivtr s terminal --status fail --latest 1 --refs
```

If status metadata is sparse:

```bash
sivtr s terminal -m "Error|error|failed|fatal|panic|Traceback|Exception|exit code|not found|External command failed|No such file or directory|permission denied|is not recognized" --latest 20 --refs
```

Expand the newest match:

```bash
sivtr zoom @last[1] -C 2 --save error_ctx --refs
sivtr show @error_ctx --full
```

## Terminal Metadata Filters

```bash
sivtr s terminal --status fail --latest 5 --refs
sivtr s terminal --exit-code 101 --latest 20 --refs
sivtr s terminal --min-duration 2s --sort duration --latest 20 -f timeline
```

## Search Terminal + AI Memory for Common Errors

```bash
sivtr s terminal -m "error|failed|panic|Traceback|Exception|exit code|could not compile|FAILED" --latest 20 --refs
sivtr s agent -m "error|failed|panic|Traceback|Exception|exit code|could not compile|FAILED" --latest 20 --refs
```

## Language Failure Queries

```bash
sivtr s terminal -m "error\\[E[0-9]+\\]|panicked|test result: FAILED|could not compile|borrow|lifetime" --latest 20 --refs
sivtr s terminal -m "TypeError|ReferenceError|TS[0-9]+|npm ERR|pnpm|vite|webpack|ELIFECYCLE|failed" --latest 20 --refs
sivtr s terminal -m "Traceback|ModuleNotFoundError|ImportError|AssertionError|pytest|FAILED|Exception" --latest 20 --refs
```

## Previous Decisions or AI Discussion

```bash
sivtr s agent -m "decision|TODO|next step|blocked|test result|passed|failed" --latest 20 --save history --refs
sivtr s @history -m "<topic>" --save topic_history --refs
sivtr zoom @topic_history[1] -C 2 --save topic_ctx --refs
sivtr show @topic_ctx --full
```

## Search Titles Instead of Content

```bash
sivtr s agent -m "workspace picker" -i session --latest 20 --refs
sivtr s terminal -m "cargo test" -i title --latest 20 --refs
```

## Provider-Specific Search

```bash
sivtr s codex -m "<query>" --latest 20 --refs
sivtr s claude -m "<query>" --latest 20 --refs
sivtr s pi -m "<query>" --latest 20 --refs
sivtr s opencode -m "<query>" --latest 20 --refs
```

## Compose Filters from the Request

Map request constraints to source selectors and filters. Keep `--match` for the content/topic being searched.

```bash
sivtr s <provider> -m "<topic>" --last <duration> --latest 20 --refs
sivtr s <provider> -m "<topic>|<related-term>|<status-term>" --last <duration> --latest 30 --refs
sivtr s agent -m "<topic>" -i <content|title|session> --cwd <path> --latest 20 --refs
```

Examples:

- "pi 中的 merge" -> source `pi`, match `merge`
- "最近两小时的终端报错" -> source `terminal`, options `--status failure --last 2h`
- "这个仓库上次的 CI 失败" -> source `terminal`, match `CI|failed`, option `--cwd <repo>`
- "标题里有 workspace picker" -> source `agent`, match `workspace picker`, option `--in session` or `--in title`

## Chaining WorkSets

Pipeline chaining uses `@` as stdin source. Intermediate commands can omit `-f`; piped stdout emits WorkSet JSON automatically.

```bash
sivtr s terminal --status fail -m "error|failed|拒绝访问" \
  | sivtr s @ -v "example|sample|sttop" -i title \
  | sivtr zoom @ -C 1 \
  | sivtr show @ -f timeline
```

Named WorkSets are useful for multi-step retrieval:

```bash
sivtr s agent -m "decision|TODO|next step" --latest 20 --save hits --refs
sivtr s @hits -m "<topic>" --save narrowed --refs
sivtr zoom @narrowed[1] -C 2 --save ctx --refs
sivtr show @ctx --full
```

## Zoom

`zoom` expands each source record to neighboring records in the same session and creates a new WorkSet.

```bash
sivtr zoom <source> -C <n> --save <name> --refs
sivtr zoom @last[1] --before 3 --after 1 -f timeline
sivtr zoom @ -C 2 | sivtr show @ --full
```

Options:

- `-C` / `--context`: set both before and after.
- `--before`: records before each source record.
- `--after`: records after each source record.
- `--save`: name the expanded WorkSet.

## Show

`show` displays any source.

```bash
sivtr show @last --full
sivtr show @last[1,3] -f timeline
sivtr show @ctx -f md
sivtr show "terminal/current/12" --full
sivtr show "pi/019e4f40/3" --full
sivtr show "codex/abc123/2/o/1" --full
```

Refs/selectors have this shape:

```text
terminal/session/dialogue[/line]
provider/session/dialogue[/line]
provider/session/dialogue/<i|o>/<part>
```

The `dialogue` / `line` segments may be concrete numbers or selector lists/ranges when used as command input, for example `3-5,7` or `5-7,10`. Output refs remain concrete anchors. Part refs use `i` (input) or `o` (output) followed by a 1-based part index.

## Token Budget

- Start with `--latest 20`.
- Use `--refs` or `-f timeline` for first-pass inspection.
- Expand at most 1-3 records before answering unless the task requires a timeline.
- Prefer exact refs and WorkSet selectors over broad repeated searches.
- If context is still missing after targeted search and expansion, ask the user.

## Copy by Ref

Copy content behind an exact ref to clipboard:

```bash
sivtr copy ref "codex/019e4f40/3/o/1"
sivtr copy ref "terminal/session_42/5" --print
sivtr copy ref "pi/abc123/2/i/1" --lines "1:10"
```

Supports `--print`, `--regex`, `--lines`, and `--cwd` options.

## Work Traversal

Explore workspace records at session, record, and part granularity:

```bash
sivtr work sessions --json
sivtr work records codex/019e4f40 --json
sivtr work parts codex/019e4f40/3 --json
sivtr work parts codex/019e4f40/3 --io output --json
```

The `work` command provides canonical refs for every level. Use `--io all|input|output` to filter part listing.

## Diagnostics

When `sivtr` commands fail or return no results, check the environment:

```bash
sivtr doctor
sivtr init show
```

Common issues:

- No terminal results → `sivtr init show` to verify hooks, then restart terminal.
- No provider results → `sivtr doctor` to check session discovery.
- Clipboard not working → `sivtr doctor` reports clipboard availability.
