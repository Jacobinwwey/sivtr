---
title: 比较命令块
description: 比较最近 shell 命令的输入、输出和完整块。
---

`sivtr diff` 比较当前 shell session 中的两个最近命令块。你在迭代构建、测试、提示词或诊断时，可以快速查看两次运行之间发生了什么变化。

必须先安装 shell 集成，`sivtr` 才有结构化命令块可比较。

```bash
sivtr init powershell
# 或：sivtr init bash / zsh / nushell
```

安装后重启 shell。

## 基本 diff

比较最新命令输出和上一次命令输出：

```bash
sivtr diff 1 2
```

Selector 按"从新到旧"计数。`1` 是最新命令块，`2` 是上一个命令块。

## 选择比较内容

默认比较输出。

```bash
sivtr diff 1 2 --output
sivtr diff 1 2 --block
sivtr diff 1 2 --input
sivtr diff 1 2 --cmd
```

| 选项 | 比较内容 |
| --- | --- |
| `--output` | 命令输出，默认值 |
| `--block` | 输入加输出 |
| `--input` | 带 prompt 的输入 |
| `--cmd` | 裸命令文本 |

同一时间只能使用一种内容模式。

## 并排视图

用两列文本视图代替 unified diff：

```bash
sivtr diff 2 1 --side-by-side
sivtr diff 3 1 --block --side-by-side
```

## 使用建议

- 刚刚重跑命令后，用 `1 2` 查看差异。
- 用 `--cmd` 确认两次复制或重试的命令是否真的不同。
- 当 prompt 或命令上下文和输出同样重要时，用 `--block`。
