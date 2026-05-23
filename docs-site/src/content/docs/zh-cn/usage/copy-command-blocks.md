---
title: 复制命令块
description: 复制最近 shell 输入、输出、命令、范围和过滤后的行。
---

`sivtr copy` 读取 shell 集成创建的结构化 shell session log，不需要打开 browser。

如果复制命令块时没有数据，先安装 shell 集成：

```bash
sivtr init powershell
# 或：sivtr init bash / zsh / nushell
```

安装后重启 shell。

## 基本模式

```bash
sivtr copy
sivtr copy in
sivtr copy out
sivtr copy cmd
```

| 命令 | 复制内容 |
| --- | --- |
| `sivtr copy` | 输入加输出 |
| `sivtr copy in` | 只复制输入，默认包含 prompt |
| `sivtr copy out` | 只复制输出 |
| `sivtr copy cmd` | 只复制裸命令 |

别名：

| 别名 | 完整命令 |
| --- | --- |
| `sivtr c` | `sivtr copy` |
| `sivtr ci` | `sivtr copy in` |
| `sivtr co` | `sivtr copy out` |
| `sivtr cc` | `sivtr copy cmd` |

## 选择最近块

Selector 相对最新命令块计数：

```bash
sivtr copy 1
sivtr copy out 2
sivtr copy in 2..4
```

`1` 是最新块，`2` 是上一个块，`2..4` 选择多个最近块。共享语法见 [Selector 和 Filter](/zh-cn/reference/selectors-and-filters/)。

## 复制后打印

用 `--print` 查看复制了什么：

```bash
sivtr copy out --print
```

文本仍然会被复制到剪贴板。

## 保留 ANSI

如果希望保留彩色终端序列，使用 `--ansi`：

```bash
sivtr copy out --ansi
```

只有当 session entry 保存了 ANSI 输出时，这个选项才有实际效果。

## 重写 prompt

输入复制模式默认保留原 prompt。用 `--prompt` 覆盖：

```bash
sivtr copy in --prompt ":"
sivtr copy --prompt ">"
```

如果 prompt 结尾没有空白，`sivtr` 会在命令前插入一个空格。

## 过滤复制文本

Filter 在选中块组装完成后运行。

```bash
sivtr copy out --regex panic
sivtr copy out --lines 10:20
sivtr copy out --lines 1,3,8:12
```

同时设置时，`--regex` 先运行，`--lines` 再作用于过滤后的结果。

## 交互式 picker

打开交互式 picker：

```bash
sivtr copy --pick
sivtr copy out --pick
sivtr copy cmd --pick
```

常用 picker 按键：

| 按键 | 动作 |
| --- | --- |
| `j` / `k` | 移动 |
| `Space` | 切换当前条目 |
| `v` | 标记范围锚点 |
| `a` | 切换全选 |
| `:` | 为下一次复制设置临时行过滤 |
| `p` | 切换预览 |
| `t` | 打开 Vim 风格 full view |
| `Enter` | 确认 |
| `Backspace` | 编辑待应用的行过滤 |
| `Esc` | 取消 |

Vim 风格 full view 支持 `[[` / `]]` 跳转块，`myy` / `myi` / `myo` / `myc` 复制，`mvv` / `mvi` / `mvo` 选择。
