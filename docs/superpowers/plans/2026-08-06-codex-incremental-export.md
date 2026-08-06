# Codex Incremental Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace unconditional Codex watch mirroring with portable, recoverable incremental replication.

**Architecture:** Native filesystem events provide debounced wakeups, while periodic recursive reconciliation remains authoritative. Local metadata observations accelerate the hot path; SHA-256 content fingerprints re-establish trust after restart or cross-platform migration.

**Tech Stack:** Rust 2021, `notify` 8.2, `sha2` 0.11, standard filesystem and channel APIs, existing `tempfile` test dependency.

## Global Constraints

- Base every implementation commit on `Ariestar/sivtr@48c72e88619010acc5c7b78fb94dbd6e025db6fb` or a clean rebase of newer `upstream/main`.
- Preserve Rust 1.95 MSRV and Linux, macOS, and Windows support.
- Keep PR scope to `sivtr codex export`; do not mix PR #70 group-mode or PR #71 release-profile work.
- Filesystem events are hints; periodic reconciliation is the correctness source.
- Never persist or transmit `SystemTime`, inode, device ID, or native path separators as content identity.
- Every commit uses Conventional Commits and `git commit -s`.
- Do not bump crate versions or changelogs.

---

## English

### File map

- Modify `Cargo.toml`: add the cross-platform watcher dependency.
- Modify `Cargo.lock`: lock transitive watcher dependencies.
- Modify `src/commands/system/codex.rs`: own export reconciliation, portable fingerprinting, append recovery, watcher wakeups, and regression tests.
- Modify `docs-site/src/content/docs/reference/cli.md`: document watch/reconcile behavior.
- Modify `docs-site/src/content/docs/zh-cn/reference/cli.md`: keep the Chinese CLI reference aligned.

No new production module is added. The synchronization invariants belong to the
existing Codex export command, and extracting a pass-through layer would add an
API without hiding another dependency or enforcing a stronger boundary.

### Task 1: Lock dependency and write regressions

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `src/commands/system/codex.rs`

**Interfaces:**
- Consumes: existing private `export_once`, `copy_session_file_atomically`, and `CodexExportArgs`.
- Produces: failing tests that define stable-pass, append, rewrite, truncation, and migration behavior.

- [ ] **Step 1: Add `notify = "8.2"` to root dependencies**

Run:

```bash
rtk cargo add notify@8.2 --manifest-path Cargo.toml
```

Expected: `Cargo.toml` and `Cargo.lock` change; resolved `notify` supports Rust 1.77 or newer.

- [ ] **Step 2: Add outcome-focused tests before implementation**

Add tests equivalent to:

```rust
#[test]
fn stable_export_does_not_republish_target() {
    let fixture = ExportFixture::new("initial\n");
    let mut checkpoints = HashMap::new();
    assert_eq!(export_once(&fixture.source_root, &fixture.target_root, 0, &mut checkpoints)?, 1);
    let before = local_file_observation(&fixture.target_file)?;
    assert_eq!(export_once(&fixture.source_root, &fixture.target_root, 0, &mut checkpoints)?, 0);
    assert_eq!(local_file_observation(&fixture.target_file)?, before);
}

#[test]
fn verified_growth_appends_only_new_bytes() {
    let fixture = ExportFixture::new("first\n");
    let published = fixture.initial_sync()?;
    fixture.append_source("second\n")?;
    let sync = synchronize_session_file(
        &fixture.source_file,
        &fixture.target_file,
        Some(published.source),
    )?;
    assert_eq!(sync.change, SessionFileChange::Appended);
    assert_eq!(fs::read(&fixture.target_file)?, b"first\nsecond\n");
}
```

Also cover:

- cold-start append with no in-memory checkpoint;
- equal bytes with deliberately different mtimes;
- same-length rewrite;
- source truncation;
- target prefix corruption;
- append delta larger than the configured memory bound.

- [ ] **Step 3: Run focused tests and confirm failure**

```bash
rtk cargo test commands::system::codex::tests --locked
```

Expected: new tests fail to compile or fail because incremental outcomes are not implemented; existing tests remain green.

### Task 2: Implement portable incremental reconciliation

**Files:**
- Modify: `src/commands/system/codex.rs`

**Interfaces:**
- Consumes: `Path`, source/target files, and an optional last published observation.
- Produces: `PublishedSessionFile { change, source }`; `export_once` stores its observation by relative path.

- [ ] **Step 1: Define explicit local and portable identities**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LocalFileObservation {
    len: u64,
    modified: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PortableContentFingerprint {
    len: u64,
    sha256: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionFileChange {
    Unchanged,
    Appended,
    Replaced,
}

struct PublishedSessionFile {
    change: SessionFileChange,
    source: LocalFileObservation,
}
```

`LocalFileObservation` must remain private and non-serializable. The portable
fingerprint is computed from bytes only.

- [ ] **Step 2: Add bounded SHA-256 and snapshot helpers**

Implement exact-byte prefix hashing with a named 64 KiB buffer. Implement
snapshot copy/read functions that stop at the source length observed at cycle
start and fail if fewer bytes are available.

```rust
fn fingerprint_file_prefix(path: &Path, byte_len: u64) -> Result<PortableContentFingerprint>;
fn read_appended_bytes(source: &Path, offset: u64, snapshot_len: u64) -> Result<Vec<u8>>;
```

The append allocation must be bounded by `MAX_INCREMENTAL_APPEND_BYTES`; larger
deltas use atomic replacement.

- [ ] **Step 3: Implement the synchronization decision table**

```rust
fn synchronize_session_file(
    source: &Path,
    target: &Path,
    last_published: Option<LocalFileObservation>,
) -> Result<PublishedSessionFile>;
```

Decision order:

1. Exact source/target local observation match -> unchanged.
2. Source grew and target equals trusted last publication -> bounded append.
3. Source grew without a checkpoint -> compare portable target and source-prefix fingerprints, then append or replace.
4. Equal lengths with metadata drift -> compare full portable fingerprints; align mtime without data rewrite when equal, otherwise replace.
5. Source shorter, prefix mismatch, or oversized delta -> atomic replacement.

- [ ] **Step 4: Thread checkpoints through reconciliation**

Change `export_once` to accept:

```rust
&mut HashMap<PathBuf, LocalFileObservation>
```

Key entries by normalized relative `PathBuf`, remove checkpoints for stale
exports, and count only `Appended`/`Replaced` as updated files.

- [ ] **Step 5: Run focused tests**

```bash
rtk cargo test commands::system::codex::tests --locked
```

Expected: all Codex export tests pass.

### Task 3: Add event-assisted watch with polling fallback

**Files:**
- Modify: `src/commands/system/codex.rs`

**Interfaces:**
- Consumes: source root and configured reconciliation interval.
- Produces: a bounded wakeup receiver that owns `notify::RecommendedWatcher`; absence of a receiver means polling fallback.

- [ ] **Step 1: Add watcher ownership and coalesced signals**

```rust
struct ExportWakeup {
    receiver: Receiver<WatchSignal>,
    _watcher: notify::RecommendedWatcher,
}

enum WatchSignal {
    Changed,
    Failed(String),
}

fn start_export_wakeup(source_root: &Path) -> Result<ExportWakeup>;
```

Use a capacity-one `sync_channel` and `try_send` so event storms cannot block
the notify callback or allocate without bound.

- [ ] **Step 2: Replace unconditional sleep with wakeup-or-timeout**

Initial export remains synchronous. In watch mode:

1. wait for event or reconciliation timeout;
2. on an event, debounce for `min(interval, 250 ms)` and drain queued events;
3. reconcile all files and stale entries;
4. warn on watcher errors, but do not stop periodic reconciliation.

- [ ] **Step 3: Add deterministic channel tests and native smoke coverage**

Test timeout, burst coalescing, and runtime error handling with injected channel
signals. Add one bounded native watcher smoke test that creates/appends a JSONL
file and observes a wakeup on all CI platforms.

- [ ] **Step 4: Run focused and full debug gates**

```bash
rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
```

Expected: zero format/clippy findings and all workspace tests pass.

### Task 4: Document behavior and produce performance evidence

**Files:**
- Modify: `docs-site/src/content/docs/reference/cli.md`
- Modify: `docs-site/src/content/docs/zh-cn/reference/cli.md`

**Interfaces:**
- Consumes: implemented CLI behavior.
- Produces: user-facing semantics for event wakeups, periodic convergence, and migration-safe recovery.

- [ ] **Step 1: Update both CLI references**

State that `--interval` is the maximum reconciliation interval, native events
may trigger earlier sync, and watcher failure falls back to polling. Do not
promise event delivery or atomic incremental append visibility.

- [ ] **Step 2: Build docs and release binary**

```bash
rtk npm run check --prefix docs-site
rtk npm run build --prefix docs-site
rtk cargo build --release --locked
```

Expected: docs checks/build and release build pass.

- [ ] **Step 3: Benchmark identical workloads**

Measure baseline and candidate binaries on the same Linux host with:

- 100 stable JSONL files totaling at least 1 GiB, 20 watch cycles;
- one growing JSONL with fixed 4 KiB appends, 100 cycles;
- `/proc/<pid>/io` `write_bytes`, elapsed time, and peak RSS.

Report raw workload, filesystem, interval, and before/after values in the PR.

### Task 5: OSS history and PR

- [ ] **Step 1: Rebase before final validation**

```bash
rtk git fetch upstream main
rtk git rebase upstream/main
```

Expected: linear history; no merge commit.

- [ ] **Step 2: Create signed logical commits**

```bash
rtk git add Cargo.toml Cargo.lock src/commands/system/codex.rs
rtk git commit -s -m "fix(codex): make watch exports incremental"
rtk git add docs-site/src/content/docs/reference/cli.md docs-site/src/content/docs/zh-cn/reference/cli.md
rtk git commit -s -m "docs(codex): document incremental watch behavior"
```

- [ ] **Step 3: Re-run every acceptance gate and inspect the full diff**

```bash
rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo build --release --locked
rtk git diff upstream/main...HEAD
```

- [ ] **Step 4: Push fork branch and open the upstream PR**

Push only to `Jacobinwwey/sivtr`; target `Ariestar/sivtr:main`. The PR body must
state the causal write-amplification mechanism, content-identity trade-off,
cross-platform fallback, regression coverage, and benchmark values. Do not
claim this exporter provides a bidirectional mobile control plane.

---

## 中文

### 文件边界

- 修改 `Cargo.toml`：加入跨平台 watcher 依赖。
- 修改 `Cargo.lock`：锁定 watcher 传递依赖。
- 修改 `src/commands/system/codex.rs`：负责 reconcile、可迁移内容指纹、追加
  恢复、事件唤醒和回归测试。
- 修改英文与中文 CLI reference：明确新 watch 语义。

不新增生产模块。同步不变量属于现有 Codex export 命令；拆出仅转发调用的层并
不能隐藏依赖或强化边界。

### 任务 1：依赖与失败回归测试

- [ ] 加入 `notify = "8.2"` 并更新 lockfile。
- [ ] 先写稳定轮次、热追加、冷启动、mtime 漂移、等长改写、截断、前缀损坏
  和超大增量测试。
- [ ] 运行 `rtk cargo test commands::system::codex::tests --locked`，确认新测试
  在实现前失败，旧测试保持通过。

### 任务 2：可迁移增量 reconcile

- [ ] 定义仅进程内使用的 `LocalFileObservation { len, modified }`。
- [ ] 定义只由字节生成的 `PortableContentFingerprint { len, sha256 }`；禁止
  serde、inode、设备 ID 或平台路径进入该身份。
- [ ] 用 64 KiB 有界 buffer 计算前缀 SHA-256，并按观测长度读取源快照。
- [ ] 实现同步决策表：元数据相等跳过；可信增长追加；冷启动先做前缀 hash；
  等长变更做完整 hash；截断/分歧/超大 delta 原子替换。
- [ ] 让 `export_once` 持有相对路径到上次发布观测的 map，并在 stale cleanup
  时删除对应 checkpoint。
- [ ] focused tests 全部通过。

### 任务 3：事件辅助 watch 与轮询 fallback

- [ ] 用容量 1 的 `sync_channel` 合并事件，callback 只做非阻塞 `try_send`。
- [ ] 初始同步后，等待事件或 reconcile timeout；事件按
  `min(interval, 250 ms)` debounce。
- [ ] watcher 初始化/运行失败只告警并降级，周期 reconcile 不停止。
- [ ] 增加 channel 的确定性测试与三平台原生 watcher smoke test。
- [ ] 通过 fmt、严格 Clippy 和 workspace tests。

### 任务 4：文档与性能证据

- [ ] 中英文 CLI reference 都说明 interval 是最大 reconcile 间隔、事件可提前
  唤醒、失败会回退轮询。
- [ ] 通过 docs check/build 与 release build。
- [ ] 同一 Linux 主机上比较基线和候选：至少 1 GiB 稳定数据集 20 轮，以及
  4 KiB 固定追加 100 轮；记录 `/proc/<pid>/io`、耗时和 peak RSS。

### 任务 5：OSS 历史与 PR

- [ ] `rtk git fetch upstream main` 后 rebase，禁止 merge commit。
- [ ] 代码/测试提交为 `fix(codex): make watch exports incremental`，文档提交为
  `docs(codex): document incremental watch behavior`；两者均使用 `-s`。
- [ ] rebase 后重新执行全部门禁并审查 `upstream/main...HEAD` 完整 diff。
- [ ] 只推送 `Jacobinwwey/sivtr`，向 `Ariestar/sivtr:main` 创建 PR；说明根因、
  指纹权衡、跨平台 fallback、回归测试与 benchmark，不宣称具备双向手机控制面。
