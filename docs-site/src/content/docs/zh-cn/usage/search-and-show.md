---
title: 搜索和展示结果
description: 搜索当前 workspace memory，并打印精确 ref。
---

`sivtr search` 查询当前 workspace 的 Agent session；如果 shell 集成已有当前终端 session log，也会包含它。`sivtr show` 打印精确 ref 背后的内容。

当交互式 picker 太重，而你需要给人类工作流、Agent prompt 或其他工具提供脚本友好的记忆时，把这两个命令组合使用。它们也是 skill 最安全的基础能力，因为可以非交互运行，并返回精确 ref。

例如，"解决终端报错" skill 可以这样开始：

```bash
sivtr search "error|failed|panic|Traceback|Exception|exit code|FAILED" --json --limit 20
sivtr copy out 1 --print
```

"最近工作 timeline" skill 可以把搜索结果和最近命令标题组合起来：

```bash
sivtr search "TODO|next step|decision|test result|passed|failed|commit|build" --json --limit 50
sivtr copy cmd 1..20 --print
```

## 搜索内容

```bash
sivtr search panic
sivtr search "workspace picker"
sivtr search "build error" --limit 20
```

默认情况下，search 使用当前目录作为 workspace，并搜索所有受支持 Agent provider 的 dialogue/content 文本。当前 shell session log 存在时，也会作为 `terminal/current` source 加入。

## 搜索 scope

```bash
sivtr search "panic" --scope content
sivtr search "release notes" --scope dialogue
sivtr search "sivtr" --scope session
```

| Scope | 搜索范围 |
| --- | --- |
| `content` | Dialogue/content 行，默认值 |
| `dialogue` | Dialogue 标题 |
| `session` | Session 标题 |

搜索查询是大小写不敏感的正则，和 workspace picker 搜索行为一致。

## Provider 过滤

```bash
sivtr search "panic" --provider all
sivtr search "panic" --provider codex
sivtr search "panic" --provider claude
sivtr search "panic" --provider opencode
sivtr search "panic" --provider pi
```

用 `all` 搜索所有受支持 provider。

## 工作区目录

覆盖用于解析 session 的工作区目录：

```bash
sivtr search "panic" --cwd /path/to/project
sivtr show claude/<session>/<dialogue> --cwd /path/to/project
```

这适合从脚本或编辑器集成中运行，而当前进程目录不是目标项目目录的情况。

## JSON 输出

其他工具需要 ref 和内容时使用 JSON：

```bash
sivtr search "build error" --json --limit 20
```

JSON 输出包含：

- query；
- scope；
- cwd；
- 总匹配数量；
- result list，其中包含 `ref`、kind、timestamp、title 和 content。

## 展示 ref

Ref 形状如下：

```text
source/session[/dialogue[/line]]
```

打印整个 session 内容：

```bash
sivtr show claude/<session>
```

打印一个 dialogue：

```bash
sivtr show claude/<session>/<dialogue>
```

打印某个 dialogue 中 1-based 的一行：

```bash
sivtr show claude/<session>/<dialogue>/<line>
```

打印终端搜索结果也使用同样结构：

```bash
sivtr show terminal/current
sivtr show terminal/current/<block>
sivtr show terminal/current/<block>/<line>
```

机器可读输出使用 JSON：

```bash
sivtr show pi/<session>/<dialogue>/<line> --json
```

## 实用循环

1. 先广泛搜索：

   ```bash
   sivtr search "panic" --provider all --json --limit 50
   ```

2. 从结果中选择你关心的 ref。
3. 打印周边 dialogue：

   ```bash
   sivtr show <source/session/dialogue>
   ```

4. 需要紧凑引用、脚本输入或后续 Agent 的上下文句柄时，再使用精确 line ref。
