---
title: 快捷键
description: Browser、命令块、picker、搜索、行过滤和 Vim view 快捷键。
---

## Browser

Browser 用于 pipe mode、run mode 和 session import。

| 按键 | 模式 | 动作 |
| --- | --- | --- |
| `j` / `Down` | Normal, Insert, Visual | 下移 |
| `k` / `Up` | Normal, Insert, Visual | 上移 |
| `h` / `Left` | Normal, Insert, Visual | 左移 |
| `l` / `Right` | Normal, Insert, Visual | 右移 |
| `0` / `Home` | Normal, Insert, Visual | 行首 |
| `^` | Normal, Insert, Visual | 第一个非空白列 |
| `$` / `End` | Normal, Insert, Visual | 行尾 |
| `Ctrl-D` | Normal, Insert, Visual | 下翻半页 |
| `Ctrl-U` | Normal, Insert, Visual | 上翻半页 |
| `Ctrl-F` / `PageDown` | Normal, Insert, Visual | 下翻页 |
| `Ctrl-B` / `PageUp` | Normal, Insert, Visual | 上翻页 |
| `gg` | Normal, Visual | 顶部 |
| `G` | Normal, Visual | 底部 |
| `H` | Normal, Visual | 视口顶部 |
| `M` | Normal, Visual | 视口中间 |
| `L` | Normal, Visual | 视口底部 |
| `/` | Normal, Insert | 搜索 |
| `n` | Normal | 下一个匹配 |
| `N` | Normal | 上一个匹配 |
| `i` | Normal | Insert mode |
| `v` | Normal, Visual | 字符选择 |
| `V` | Normal, Visual | 行选择 |
| `Ctrl-V` | Normal, Visual | 块选择 |
| `o` | Visual | 交换选择锚点 |
| `y` | Visual | 复制选择 |
| `e` | Normal, Visual | 打开编辑器 |
| `Esc` | Insert, Visual, Search | 取消当前模式 |
| `q` | Normal | 退出 |

## 命令块导航

仅当 buffer 有解析后的命令块时可用。

| 按键 | 动作 |
| --- | --- |
| `[[` | 上一个命令块 |
| `]]` | 下一个命令块 |
| `myy` | 复制当前命令块 |
| `myi` | 复制当前命令输入 |
| `myo` | 复制当前命令输出 |
| `myc` | 复制当前裸命令 |
| `mvv` | 选择当前命令块 |
| `mvi` | 选择当前命令输入 |
| `mvo` | 选择当前命令输出 |

## Workspace picker

Workspace picker 用于 AI session picker 和命令块 picker。

| 按键 | 动作 |
| --- | --- |
| `0` | 聚焦 Source 面板 |
| `1` | 聚焦 Sessions 面板 |
| `2` | 聚焦 Dialogues 面板 |
| `3` | 聚焦 Content 面板 |
| `j` / `Down` | 在聚焦面板中下移 |
| `k` / `Up` | 在聚焦面板中上移 |
| `h` / `Left` | 聚焦前一个面板 |
| `l` / `Right` | 聚焦后一个面板 |
| `Space` | 切换当前 source、session 或 dialogue |
| `a` | 按面板不同，选择所有 source 或切换所有 dialogue |
| `g` | 在 Source 面板中选择 agent sources |
| `t` | 在 Source 面板选择 terminal source，或在其他位置打开 Vim-style full view |
| `v` | Range-select dialogues，或在 Content 面板开始可视文本选择 |
| `:` | 为下一次复制启动临时行过滤 |
| `i` | 复制输入/问题 |
| `o` | 复制输出/回答 |
| `y` | 复制输入 + 输出块 |
| `c` | 在可用时复制裸命令 |
| `Enter` | 进入面板或复制当前选择 |
| `Ctrl-D` / `PageDown` | Content 下滚 |
| `Ctrl-U` / `PageUp` | Content 上滚 |
| `r` | 在 Content 面板切换 raw/read content mode |
| `z` | 当前面板全屏切换 |
| `/` | 搜索所有 session |
| `?` | 打开帮助 |
| `q` / `Esc` | 取消或返回 |

## Workspace 搜索

| 按键 | 动作 |
| --- | --- |
| `/` | 打开搜索输入 |
| `Enter` | 接受搜索输入 |
| `Esc` | 清除或关闭搜索 |
| `Backspace` | 输入打开时编辑 query |
| `n` | 下一个匹配 |
| `N` | 上一个匹配 |

搜索前缀：

| 前缀 | 范围 |
| --- | --- |
| 无 | Content |
| `#` | Dialogue 标题 |
| `>` | Session 标题 |

## 行过滤输入

在 workspace picker 中按 `:` 打开。

| 按键 | 动作 |
| --- | --- |
| 数字、`,`、`:` | 构建 1-based 行 spec |
| `Backspace` | 编辑待应用过滤 |
| `Enter` | 应用到下一次复制 |
| `Esc` | 清除/取消 |

示例：`2:8`、`1,3,8:12`。

## Vim-style full view

在 picker 中按 `t` 打开。

| 按键 | 动作 |
| --- | --- |
| `[[` | 上一个块 |
| `]]` | 下一个块 |
| `myy` | 复制块 |
| `myi` | 复制输入 |
| `myo` | 复制输出 |
| `myc` | 复制命令 |
| `mvv` | 选择块 |
| `mvi` | 选择输入 |
| `mvo` | 选择输出 |
| `T` | 在可用时切换 alternate tool view |
| `p`, `q`, `Esc` | 返回 picker |
