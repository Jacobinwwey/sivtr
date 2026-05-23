---
title: 历史记录
description: 列出、搜索、展示并保留保存过的终端捕获。
---

`sivtr` 会把捕获到的终端输出存入本地 SQLite history 数据库，并使用 FTS5 搜索。History 面向更长期的终端捕获，它独立于每个 shell 的 session log 和 AI agent transcript。

## 列出最近条目

```bash
sivtr history
sivtr history list
sivtr history list --limit 50
```

输出包含 entry id、时间戳、命令和内容预览。

## 搜索 history

```bash
sivtr history search "panic"
sivtr history search "failed assertion" --limit 10
```

搜索使用 history 全文索引。拿到 id 后可以配合 `history show`。

这不同于 `sivtr search`：后者搜索当前 workspace 的 Agent session，并在 shell 集成有数据时包含当前终端 session log。

## 展示条目

```bash
sivtr history show 42
```

详情视图会先打印元数据，再打印保存的内容：

- id；
- timestamp；
- command；
- source；
- host；
- content。

## 保留策略

History 保留策略由配置控制：

```toml
[history]
auto_save = true
max_entries = 0
```

`max_entries = 0` 表示不限制数量。

禁用自动保存：

```toml
[history]
auto_save = false
```

## 什么时候用 history，什么时候用 session log

| 需求 | 使用 |
| --- | --- |
| 搜索较早保存的终端输出 | `sivtr history search` |
| 复制最近 shell 命令输出 | `sivtr copy out` |
| 浏览当前 shell 的结构化块 | `sivtr import` |
| 搜索当前 workspace 的终端和 Agent memory | `sivtr search` |
