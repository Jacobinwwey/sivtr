---
title: 安装
description: 使用 Cargo 安装 sivtr，并设置 shell 集成。
---

`sivtr` 以 Cargo 包发布，源码位于 [github.com/Ariestar/sivtr](https://github.com/Ariestar/sivtr)。

## 要求

- Rust 和 Cargo
- 一个受支持的终端
- 当前平台可用的剪贴板能力

可选：

- `nvim`、`vim` 或 `vi`，用于部分 copy 工作流中的 Vim picker view
- 可修改 PowerShell、Bash、Zsh 或 Nushell profile，用于会话记录

## 用 Cargo 安装

从 crates.io 安装最新发布版本：

```bash
cargo install sivtr
```

验证二进制：

```bash
sivtr --version
sivtr --help
```

## 从源码安装

克隆仓库：

```bash
git clone https://github.com/Ariestar/sivtr.git
cd sivtr
```

在仓库根目录运行：

```bash
cargo install --path .
```

## 更新

更新已发布包：

```bash
cargo install sivtr --force
```

或在本地 checkout 拉取后重新安装：

```bash
git pull
cargo install --path . --force
```

Cargo 会替换之前安装的二进制。

## Shell 集成

Shell 集成会记录最近的命令块，让 `sivtr copy`、`sivtr import` 和命令块导航有结构化数据可用。

为你的 shell 安装 hook：

```bash
sivtr init powershell
sivtr init bash
sivtr init zsh
sivtr init nushell
```

安装后重启终端。

Hook 会写入按进程区分的 session log：

- Windows PowerShell 和 PowerShell 7 使用 `%APPDATA%\sivtr\session_<pid>.log`。
- Bash 和 Zsh 使用 `$XDG_STATE_HOME/sivtr/session_<pid>.log` 或 `~/.local/state/sivtr/session_<pid>.log`。
- Nushell 使用自己的 config/state 区域中的 `sivtr` session 文件。

## 配置文件

创建默认配置：

```bash
sivtr config init
```

查看路径和当前内容：

```bash
sivtr config show
```

用配置的编辑器打开：

```bash
sivtr config edit
```

完整配置项见[配置文件](/zh-cn/reference/config-file/)。
