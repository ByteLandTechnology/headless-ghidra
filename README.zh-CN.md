# Headless Ghidra 技能族

Headless Ghidra 是一组供 agent 使用的 Ghidra 逆向工程技能。使用者安装技能族后，只需要用自然语言让 agent 使用 `headless-ghidra`；随附的 `ghidra-agent-cli` 会由技能自动调用，不需要手动安装、构建或直接运行。

语言版本：[English](./README.md) | [日本語](./README.ja-JP.md)

## 安装

Codex 推荐使用内置的 `$skill-installer`：

```text
$skill-installer install all skills from https://github.com/ByteLandTechnology/headless-ghidra
```

它应安装 7 个同级技能：`headless-ghidra`、5 个 P0-P4 阶段技能，以及随附工具技能 `ghidra-agent-cli`。安装后重启 Codex，让新技能生效。

使用 `skills` CLI 可一次把本技能族全部技能安装到所有支持的 agent：

```sh
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --all
```

这里的 `--all` 等同于 `--skill '*' --agent '*' --yes`。

如果只想把全部技能安装到单个 agent：

```sh
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --agent codex --skill '*' --yes
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --agent claude-code --skill '*' --yes
```

## 使用

前置条件：

- 本机已安装 Ghidra。
- 目标二进制位于 agent 可读的工作区路径。
- Frida 可选，只在需要运行时观察时使用。

开始新的分析：

```text
请使用 headless-ghidra 技能分析 ./sample-target。从 P0 intake 开始，
选择稳定的 target id，并在每个阶段 gate 后暂停，让我审查产物。
```

继续已有目标：

```text
继续同一个目标，进入 P1 baseline。
显示当前 pipeline state 和我需要审查的产物。
进入 P2 evidence，但证据不足时不要直接分类第三方代码。
对选定热路径函数运行下一轮 P3/P4 迭代。
```

运行产生的内容会写入当前工作区的 `targets/<target-id>/` 和 `artifacts/<target-id>/`，不要写回已安装的技能目录。

## 文档地图

| 你想要...                             | 阅读                                                                                                                                                            |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 安装技能族并开始一次分析              | 本 README                                                                                                                                                       |
| 理解入口技能如何路由工作              | [编排技能指南](./headless-ghidra/README.zh-CN.md)                                                                                                               |
| 运行或审查某个具体阶段                | 下方对应的 P0-P4 阶段 README                                                                                                                                    |
| 选择 walkthrough、playbook 或脚本指南 | [示例与指南](./headless-ghidra/examples/README.zh-CN.md)                                                                                                        |
| 决定先分析什么                        | [分析选择手册](./headless-ghidra/examples/analysis-selection-playbook.zh-CN.md)                                                                                 |
| 查看完整分析叙事                      | [逆向工程 walkthrough](./headless-ghidra/examples/reverse-engineering-walkthrough.zh-CN.md)                                                                     |
| 编写或审查自定义 Ghidra 脚本          | [Ghidra 脚本编写](./headless-ghidra/examples/ghidra-script-authoring.md) 和 [Ghidra 脚本审查清单](./headless-ghidra/examples/ghidra-script-review-checklist.md) |
| 调试 agent 命令语法或输出             | [CLI 工具参考](./ghidra-agent-cli/README.zh-CN.md)                                                                                                              |

## 阶段

| 阶段 | README                                                               | 用途                                         |
| ---- | -------------------------------------------------------------------- | -------------------------------------------- |
| P0   | [Intake](./headless-ghidra-intake/README.zh-CN.md)                   | 确认目标、初始化工作区、设置范围。           |
| P1   | [Baseline](./headless-ghidra-baseline/README.zh-CN.md)               | 导入 Ghidra、导出 baseline、记录运行时证据。 |
| P2   | [Evidence](./headless-ghidra-evidence/README.zh-CN.md)               | 识别第三方代码和证据来源。                   |
| P3   | [Discovery](./headless-ghidra-discovery/README.zh-CN.md)             | 补全名称、签名、类型、常量和字符串。         |
| P4   | [Batch Decompile](./headless-ghidra-batch-decompile/README.zh-CN.md) | 对选定函数应用元数据并反编译。               |
