# ghidra-agent-cli

`ghidra-agent-cli` 是 Headless Ghidra 技能族随附的辅助工具。各技能会调用它来创建目标工作区、检查二进制、运行受支持的 Ghidra 和 Frida 操作、管理 YAML 产物，并检查阶段门控。

语言版本：[English](./README.md) | [日本語](./README.ja-JP.md)

正常使用时，请从技能族 README 开始，并让 agent 使用 `headless-ghidra` 技能。本文是给 agent 工具语义和排障场景使用的参考；普通技能工作流中，最终使用者不需要手动安装、构建或运行这个 CLI。

## 辅助工具运行前置条件

以下条件适用于已安装技能调用该辅助工具时。

- Node.js >= 18，用于 npm wrapper。
- 本地 Ghidra 安装，可由 `ghidra-agent-cli ghidra discover` 发现，或通过 `GHIDRA_INSTALL_DIR` 配置。
- 可选 Frida 安装，用于 `frida *` 命令。

## 可用性

CLI 会随技能族一起安装，并由已安装技能目录中的阶段技能调用。如果 agent 报告 `ghidra-agent-cli` 不可用，请重新安装或刷新整个技能族，确保 `headless-ghidra`、各阶段技能和 `ghidra-agent-cli` 仍然是同级目录。

## 调用方式

```sh
ghidra-agent-cli [GLOBAL FLAGS] <COMMAND> [COMMAND FLAGS]
```

全局参数：

- `--format yaml|json|toml` - 输出格式，默认 `yaml`。
- `--target <id>` - 目标选择器。
- `--workspace <path>` - 工作区根路径。
- `--config-dir <PATH>` - 覆盖配置目录。
- `--data-dir <PATH>` - 覆盖数据目录。
- `--state-dir <PATH>` - 覆盖状态目录。
- `--cache-dir <PATH>` - 覆盖缓存目录。
- `--log-dir <PATH>` - 覆盖日志目录。
- `--lock-timeout <SECS>` - 获取工作区锁的超时时间，默认 `30`。
- `--no-wait` - 不等待工作区锁。
- `--help` - 显示帮助。

大多数目标相关命令需要 `--target <id>`，或先通过 `context use` 设置活动上下文。

## 输出与错误

成功命令会按所选格式输出结构化信封：

```yaml
status: ok
message: "<summary>"
data: <structured payload>
```

错误也是结构化的：

```json
{
  "code": "E_ERROR",
  "message": "description of what went wrong",
  "source": "main",
  "format": "json"
}
```

已知错误码包括 `E_ERROR`、`E_GATE_FAILED` 和 `E_LOCK_TIMEOUT`。

## 命令组

| 命令组                                                                          | 用途                                                               |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `workspace`                                                                     | 初始化目标工作区并管理阶段状态。                                   |
| `scope`                                                                         | 管理 `scope.yaml`。                                                |
| `functions`、`callgraph`、`types`、`vtables`、`constants`、`strings`、`imports` | 读取和整理 baseline 元数据。                                       |
| `third-party`                                                                   | 记录第三方库、显式无第三方审查、原始源码和函数分类。               |
| `runtime`                                                                       | 管理 `runtime/run-manifest.yaml` 和 `runtime/run-records/*.yaml`。 |
| `hotpath`                                                                       | 管理 `runtime/hotpaths/call-chain.yaml`。                          |
| `metadata`                                                                      | 管理 P3 元数据，例如重命名和签名。                                 |
| `substitute`                                                                    | 管理 P4 替换记录。                                                 |
| `git-check`                                                                     | 当 gate 要求时，检查所需产物是否已准备好接受审查。                 |
| `execution-log`                                                                 | 追加和查看执行记录。                                               |
| `progress`                                                                      | 旧进度 YAML 的辅助命令。                                           |
| `gate`                                                                          | 运行聚合门控检查并查看门控报告。                                   |
| `ghidra`                                                                        | 发现 Ghidra、导入/分析、导出 baseline、应用元数据、反编译和重建。  |
| `frida`                                                                         | 设备、捕获、比较、追踪、运行和调用辅助命令。                       |
| `inspect`                                                                       | 只读二进制检查辅助命令。                                           |
| `context`                                                                       | 活动目标上下文辅助命令。                                           |
| `paths`                                                                         | 显示解析后的工作区和运行时路径。                                   |
| `validate`、`help`                                                              | 验证和帮助入口。                                                   |

## 工作区布局

```text
targets/<target-id>/
└── ghidra-projects/

artifacts/<target-id>/
├── pipeline-state.yaml
├── scope.yaml
├── intake/
├── baseline/
│   ├── functions.yaml
│   ├── callgraph.yaml
│   ├── types.yaml
│   ├── vtables.yaml
│   ├── constants.yaml
│   ├── strings.yaml
│   └── imports.yaml
├── runtime/
│   ├── project/
│   ├── fixtures/
│   ├── run-manifest.yaml
│   ├── run-records/
│   └── hotpaths/call-chain.yaml
├── third-party/
│   ├── identified.yaml
│   ├── pristine/<library>@<version>/
│   └── compat/<library>@<version>/
├── metadata/
│   ├── renames.yaml
│   ├── signatures.yaml
│   ├── types.yaml
│   ├── constants.yaml
│   ├── strings.yaml
│   └── apply-records/
├── substitution/
│   ├── template/
│   ├── next-batch.yaml
│   └── functions/<fn_id>/
│       ├── capture.yaml
│       └── substitution.yaml
├── gates/
└── scripts/
```

`workspace init` 会创建基础工作区结构，并初始化 `pipeline-state.yaml` 和 `scope.yaml`。后续阶段通过 CLI 命令填充其他目录。

## 常见辅助命令

```sh
# 发现前置条件
ghidra-agent-cli ghidra discover

# 创建目标工作区
ghidra-agent-cli workspace init --target libfoo --binary ./libfoo.so
ghidra-agent-cli --target libfoo scope set --mode full

# 导出 baseline 证据
ghidra-agent-cli --target libfoo ghidra import
ghidra-agent-cli --target libfoo ghidra auto-analyze
ghidra-agent-cli --target libfoo ghidra export-baseline
ghidra-agent-cli --target libfoo functions list

# 运行时和热路径证据
ghidra-agent-cli --target libfoo runtime record --key entrypoint --value 0x401000
ghidra-agent-cli --target libfoo hotpath add --addr 0x401000 --reason runtime

# 元数据补全与 Ghidra 应用
ghidra-agent-cli --target libfoo metadata enrich-function \
  --addr 0x401000 \
  --name main \
  --prototype 'int(void)'
ghidra-agent-cli --target libfoo ghidra apply-renames
ghidra-agent-cli --target libfoo ghidra verify-renames
ghidra-agent-cli --target libfoo ghidra apply-signatures
ghidra-agent-cli --target libfoo ghidra verify-signatures

# 选定反编译和替换记录
ghidra-agent-cli --target libfoo ghidra decompile --fn-id fn_001 --addr 0x401000
ghidra-agent-cli --target libfoo substitute add \
  --fn-id fn_001 \
  --addr 0x401000 \
  --replacement 'return 0;'

# 门控检查
ghidra-agent-cli --target libfoo gate check --phase P1
ghidra-agent-cli --target libfoo gate list
```
