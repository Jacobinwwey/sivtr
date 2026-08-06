# Codex Incremental Export Design

## English

**Status:** Approved on 2026-08-06

### Context

`sivtr codex export --watch` currently scans every Codex JSONL file and
atomically republishes every target on every polling pass. The source files are
append-heavy, but the exporter treats them as unrelated immutable snapshots.
An idle mirror therefore writes `O(P * S)` bytes, where `P` is the number of
polls and `S` is the mirrored byte count. A growing active file is repeatedly
copied in full, so cumulative writes approach `O(S^2 / delta)` for fixed append
size `delta`.

The observed production failure is a consequence of this mechanism, not a
Tailscale or NVMe transport failure. The exporter produced approximately 627 TB
of block writes on the affected host. The SSD reports 1.67 PB written and 81%
life used.

### Decision

Make watch export an event-assisted, periodically reconciled incremental
replicator in one focused PR.

1. Native filesystem events wake the exporter quickly and are debounced.
2. A periodic recursive reconciliation remains the source of correctness.
3. Unchanged files are rejected by a metadata fast path without data writes.
4. Verified append-only growth is copied from the previous target length.
5. Truncation, same-length rewrite, or prefix divergence triggers atomic full
   replacement.
6. Watcher setup/runtime failure degrades to periodic polling with a warning.

The event stream is a hint, not a journal. This avoids depending on backend-
specific delivery guarantees from inotify, FSEvents, or ReadDirectoryChangesW.

### Identity and migration

Two different concepts must not be called one fingerprint:

- `LocalFileObservation { len, modified }` is an in-process optimization. It is
  cheap and restart-safe on one filesystem, but it is not serialized and is
  never used as a device or network identity.
- `PortableContentFingerprint { len, sha256 }` establishes byte identity after
  restart, migration, or metadata drift. It excludes inode numbers, path
  separators, platform timestamps, and filesystem IDs.

The target file is the durable checkpoint. No sidecar database or machine-
specific inode state is required. On a cold start, the exporter hashes the
target and the same-length source prefix before resuming an append. This costs
`O(S)` reads once but avoids `O(S)` writes. Future group/share migration work
may serialize the portable fingerprint, but this PR deliberately does not add
a wire or database schema before PR #70's compatibility and migration model is
settled.

PR #70 and PR #71 are compatibility inputs, not implementation bases. PR #70
adds replicated group/share state and still has unresolved authorization and
migration semantics. PR #71 changes only the release profile. This change is
based on `main`, adds no protocol message, and does not couple file identity to
either branch.

### Synchronization invariants

- A published target is always a prefix snapshot of the source observed for
  that cycle.
- A known target/source metadata match performs no content or metadata write.
- An append is allowed only when the target is a verified prefix of the source.
- The process-local checkpoint permits an `O(1)` hot-path prefix decision only
  while target length and mtime still match the last published source.
- A cold-start append requires portable prefix verification.
- Same-length content changes are detected by portable content fingerprints.
- Source truncation and prefix mismatch use the existing atomic replacement
  path.
- Stale-file cleanup still runs during periodic reconciliation and preserves
  `--limit` behavior.

Codex JSONL readers already tolerate an incomplete trailing record. Incremental
append may expose a transient partial tail while a source write is in progress;
the next event or reconciliation completes it. Atomic replacement remains the
fallback for non-append changes.

### Watch loop

Use `notify` 8.2.x. It supports the repository's Rust 1.95 MSRV and the three CI
platforms. A bounded channel coalesces redundant events so callback pressure
cannot grow memory without bound. The debounce duration is the smaller of the
configured reconciliation interval and 250 ms.

`--interval` and `--interval-ms` become the maximum periodic reconciliation
interval. Existing polling behavior remains available when native watching is
unavailable. This semantic change improves latency without removing the
existing convergence bound.

### Failure recovery

- Lost/coalesced event: periodic reconciliation discovers the change.
- Watcher initialization failure: warn once and poll.
- Watcher runtime error: warn and continue; reconciliation remains active.
- Process crash during append: the target remains a source prefix; cold-start
  prefix verification resumes from the target length.
- Source grows during a cycle: copy only the snapshotted length; the next cycle
  copies the remainder.
- Source truncates during a read: fail the cycle visibly and retry on the next
  event/reconciliation.
- Target tampering or migration metadata drift: portable content verification
  either re-establishes trust or forces atomic replacement.

### Complexity and resource bounds

Let `N` be session count, `S` total mirrored bytes, and `delta` newly appended
bytes.

- Idle event wakeup: no reconciliation work after coalescing unrelated events.
- Periodic reconciliation: `O(N log N)` for current collection/sort plus `O(N)`
  metadata reads, zero content writes.
- Hot append: `O(delta)` reads, allocation, and writes.
- Cold restart/migration append: `O(S)` reads for prefix verification plus
  `O(delta)` writes.
- Rewrite/truncation: `O(S)` full atomic replacement, as required for
  correctness.
- Event queue memory: `O(1)`; append buffer: bounded. Oversized deltas use the
  atomic replacement path rather than unbounded allocation.

### Test and acceptance strategy

- Regression: two stable passes report zero updated files and preserve target
  metadata.
- Append: verified growth returns the append outcome and writes only the suffix.
- Cold start: metadata drift with identical content performs no data rewrite.
- Migration: content fingerprints match despite different mtimes.
- Rewrite/truncation/prefix divergence: atomic fallback restores exact bytes.
- Watch channel: event bursts coalesce; timeout still triggers reconciliation.
- Native watcher smoke: source creation/append wakes the loop on Linux, macOS,
  and Windows CI without making event delivery the correctness oracle.
- Full gates: fmt, strict Clippy, workspace tests, release build, and a Linux
  steady-state/write-amplification benchmark.

### Rejected alternatives

- Metadata-only persisted fingerprint: fast but incorrect after cross-platform
  migration or timestamp preservation failures.
- Native event stream without reconciliation: misses changes after overflow,
  watcher restart, network filesystems, and backend-specific rename behavior.
- Reflink/copy-on-write snapshots: efficient on selected filesystems but not a
  portable contract.
- Privileged read broker: eliminates mirroring and is the stronger long-term
  architecture, but it changes trust, protocol, and deployment boundaries and
  must be a separate RFC/feature PR.

### Unstated critical question

The mirror is a read-only memory projection, not a bidirectional agent control
plane. It cannot provide reply routing, exactly-once delivery, session
ownership, or disconnected command replay. Replacing Paseo or MindFS for mobile
message handling requires those contracts independently of this exporter.

---

## 中文

**状态：** 2026-08-06 已批准

### 背景

当前 `sivtr codex export --watch` 每轮都会扫描全部 Codex JSONL，并对每个
目标文件执行完整的原子重新发布。源文件主要是追加写，但导出器把每个版本
都视为无关的不可变快照。空闲镜像的写入量因此为 `O(P * S)`，其中 `P` 是
轮询次数，`S` 是镜像总字节数；活跃文件按固定增量 `delta` 增长时，累计写入
趋近 `O(S^2 / delta)`。

生产故障来自该代码机制，而不是 Tailscale 或 NVMe 传输。受影响主机上该
导出进程约产生 627 TB 块写入；SSD SMART 显示累计写入 1.67 PB、寿命已使用
81%。

### 决策

在一个聚焦 PR 中，把 watch export 改为“事件辅助、周期校准”的增量复制器：

1. 原生文件事件用于快速唤醒，并进行 debounce。
2. 周期递归 reconcile 仍是正确性的最终来源。
3. 未变化文件通过元数据热路径跳过，不产生数据写入。
4. 只有经过验证的追加增长才从旧目标长度继续复制。
5. 截断、等长改写或前缀分歧回退到原子完整替换。
6. watcher 初始化或运行失败时告警，并优雅降级为周期轮询。

事件流只是提示，不是日志，因此不会依赖 inotify、FSEvents 或
ReadDirectoryChangesW 各自不同的交付保证。

### 身份与迁移

必须区分两类概念，不能笼统称为一个“指纹”：

- `LocalFileObservation { len, modified }` 仅用于进程内热路径。它便宜，适合
  单个文件系统内重启，但不序列化，也绝不作为设备或网络身份。
- `PortableContentFingerprint { len, sha256 }` 用于重启、迁移或元数据漂移后
  重新确认字节身份。它不包含 inode、路径分隔符、平台时间戳或文件系统 ID。

目标文件本身就是持久 checkpoint，不引入 sidecar 数据库或机器相关 inode
状态。冷启动时，在恢复追加前分别计算目标与等长源前缀的 SHA-256。其一次性
代价是 `O(S)` 读取，但避免 `O(S)` 写入。未来 group/share 迁移可以序列化可
迁移内容指纹；本 PR 不会在 PR #70 的兼容与迁移模型尚未稳定前增加 wire 或
数据库 schema。

PR #70 和 #71 是兼容性输入，不是本实现的基线。#70 增加 group/share 状态
复制，但授权和迁移语义仍有未解决问题；#71 只修改 release profile。本改动
基于 `main`，不增加协议消息，也不把文件身份耦合到这两个分支。

### 同步不变量

- 已发布目标始终是该轮观测源文件的前缀快照。
- 目标与已知源元数据一致时，不产生内容或元数据写入。
- 只有确认目标是源前缀时才允许追加。
- 只有当目标长度和 mtime 仍等于上次发布源时，进程内 checkpoint 才允许
  `O(1)` 的热路径前缀判断。
- 冷启动追加必须进行可迁移前缀校验。
- 等长内容改写通过可迁移内容指纹检测。
- 源截断和前缀不一致使用现有原子替换路径。
- 周期 reconcile 继续执行 stale cleanup，并保持 `--limit` 语义。

Codex JSONL 读取器已经容忍未完成的尾记录。增量追加可能在源写入过程中短暂
暴露不完整尾部；下一次事件或 reconcile 会补齐。非追加变更仍走原子替换。

### Watch 循环

采用 `notify` 8.2.x；它兼容项目 Rust 1.95 MSRV 以及三类 CI 平台。使用有界
channel 合并冗余事件，避免 callback 压力造成无界内存增长。Debounce 时长取
配置 reconcile 间隔与 250 ms 的较小值。

`--interval` 和 `--interval-ms` 定义周期 reconcile 的最大间隔。原生 watcher
不可用时仍保留原有轮询行为。该语义提升响应速度，同时不取消既有收敛上界。

### 故障恢复

- 事件丢失或合并：周期 reconcile 发现变化。
- watcher 初始化失败：告警一次并轮询。
- watcher 运行错误：告警并继续，reconcile 仍然工作。
- 追加中进程崩溃：目标仍是源前缀；冷启动前缀校验后从目标长度恢复。
- 单轮中源继续增长：只复制快照长度，下一轮复制剩余部分。
- 读取中源被截断：该轮显式失败，下一事件或 reconcile 重试。
- 目标被篡改或迁移导致元数据漂移：内容校验重新建立信任，或强制原子替换。

### 复杂度与资源上界

设 `N` 为 session 数量，`S` 为镜像总字节数，`delta` 为新增字节数：

- 空闲事件唤醒：无关事件合并后不执行 reconcile。
- 周期 reconcile：现有收集/排序为 `O(N log N)`，元数据读取为 `O(N)`，内容
  写入为零。
- 热追加：`O(delta)` 读取、分配和写入。
- 冷启动/迁移追加：`O(S)` 前缀校验读取，加 `O(delta)` 写入。
- 改写/截断：为保证正确性执行 `O(S)` 原子完整替换。
- 事件队列内存 `O(1)`；追加 buffer 有上界。过大增量走原子替换，避免无界
  分配。

### 测试与验收

- 回归：连续两次稳定同步返回零更新，并保持目标元数据不变。
- 追加：已验证增长返回 append outcome，且只写后缀。
- 冷启动：内容一致但元数据漂移时不重写数据。
- 迁移：mtime 不同仍能通过内容指纹确认一致。
- 改写、截断、前缀分歧：原子 fallback 恢复精确字节。
- Watch channel：事件风暴被合并；超时仍触发 reconcile。
- 原生 watcher smoke：Linux、macOS、Windows CI 上创建/追加能唤醒，但事件
  交付不作为唯一正确性依据。
- 完整门禁：fmt、严格 Clippy、workspace tests、release build，以及 Linux
  稳态/写放大基准。

### 拒绝的替代方案

- 持久化纯元数据指纹：速度快，但跨平台迁移或时间戳保留失败后不正确。
- 只依赖原生事件、不做 reconcile：无法处理 overflow、watcher 重启、网络
  文件系统和平台 rename 差异。
- reflink/CoW snapshot：部分文件系统高效，但不能成为跨平台契约。
- 特权读取代理：从长期看可彻底消除镜像，但会改变信任、协议和部署边界，
  必须作为独立 RFC/feature PR。

### 未明说的核心问题

该镜像是只读记忆投影，不是双向 agent 控制面。它不能提供回复路由、
exactly-once 投递、session 所有权或断线命令重放。要替代 Paseo/MindFS 的手机
消息处理能力，必须另行定义这些契约，不能由 JSONL 镜像能力推导得到。
