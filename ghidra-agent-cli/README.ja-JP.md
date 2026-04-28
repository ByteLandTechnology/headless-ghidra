# ghidra-agent-cli

`ghidra-agent-cli` は Headless Ghidra skill family に同梱される helper です。各 skill が target workspace の作成、binary inspection、対応済み Ghidra/Frida operation、YAML artifact 管理、phase gate check のために呼び出します。

言語: [English](./README.md) | [简体中文](./README.zh-CN.md)

通常の利用では skill family README から始め、agent に `headless-ghidra` skill を使うよう依頼してください。この README は command semantics と troubleshooting のための agent tool reference です。通常の skill workflow で利用者がこの CLI を手動で install、build、run する必要はありません。

## Helper Runtime Prerequisites

これらは installed skill が helper を呼び出すときの前提条件です。

- npm wrapper 用の Node.js >= 18。
- `ghidra-agent-cli ghidra discover` で検出できる、または `GHIDRA_INSTALL_DIR` で指定したローカル Ghidra。
- `frida *` コマンド用の Frida。これは任意です。

## Availability

CLI は skill family と一緒にインストールされ、installed skill directory から呼び出されます。agent が `ghidra-agent-cli` を利用できないと報告した場合は、`headless-ghidra`、phase skills、`ghidra-agent-cli` が sibling directory として揃うよう、skill family 全体を再インストールまたは refresh してください。

## 呼び出し

```sh
ghidra-agent-cli [GLOBAL FLAGS] <COMMAND> [COMMAND FLAGS]
```

グローバルフラグ:

- `--format yaml|json|toml` - 出力形式。既定は `yaml`。
- `--target <id>` - ターゲットセレクタ。
- `--workspace <path>` - ワークスペース root path。
- `--config-dir <PATH>` - config directory の上書き。
- `--data-dir <PATH>` - data directory の上書き。
- `--state-dir <PATH>` - state directory の上書き。
- `--cache-dir <PATH>` - cache directory の上書き。
- `--log-dir <PATH>` - log directory の上書き。
- `--lock-timeout <SECS>` - ワークスペース lock 取得 timeout。既定は `30`。
- `--no-wait` - ワークスペース lock を待たない。
- `--help` - help を表示。

多くの target-specific command には `--target <id>`、または `context use` で設定した active context が必要です。

## 出力とエラー

成功した command は、選択した形式で構造化エンベロープを出力します。

```yaml
status: ok
message: "<summary>"
data: <structured payload>
```

エラーも構造化されています。

```json
{
  "code": "E_ERROR",
  "message": "description of what went wrong",
  "source": "main",
  "format": "json"
}
```

既知の error code には `E_ERROR`、`E_GATE_FAILED`、`E_LOCK_TIMEOUT` があります。

## コマンドグループ

| グループ                                                                        | 用途                                                                                          |
| ------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `workspace`                                                                     | ターゲットワークスペースを初期化し、フェーズ状態を管理する。                                  |
| `scope`                                                                         | `scope.yaml` を管理する。                                                                     |
| `functions`、`callgraph`、`types`、`vtables`、`constants`、`strings`、`imports` | baseline metadata を読み取り、整理する。                                                      |
| `third-party`                                                                   | サードパーティライブラリ、明示的な「該当なし」レビュー、pristine source、関数分類を記録する。 |
| `runtime`                                                                       | `runtime/run-manifest.yaml` と `runtime/run-records/*.yaml` を管理する。                      |
| `hotpath`                                                                       | `runtime/hotpaths/call-chain.yaml` を管理する。                                               |
| `metadata`                                                                      | rename や signature などの P3 metadata を管理する。                                           |
| `substitute`                                                                    | P4 substitution record を管理する。                                                           |
| `git-check`                                                                     | gate が要求する場合に、必要な artifact が review-ready か確認する。                           |
| `execution-log`                                                                 | 実行記録を追加、確認する。                                                                    |
| `progress`                                                                      | legacy progress YAML 用の helper。                                                            |
| `gate`                                                                          | 集約 gate check を実行し、gate report を確認する。                                            |
| `ghidra`                                                                        | Ghidra 検出、import/analyze、baseline export、metadata apply、decompile、rebuild を行う。     |
| `frida`                                                                         | device、capture、compare、trace、run、invoke helper。                                         |
| `inspect`                                                                       | read-only binary inspection helper。                                                          |
| `context`                                                                       | active target context helper。                                                                |
| `paths`                                                                         | 解決済み workspace/runtime path を表示する。                                                  |
| `validate`、`help`                                                              | validation と help surface。                                                                  |

## ワークスペース配置

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

`workspace init` は基本ワークスペース構造を作成し、`pipeline-state.yaml` と `scope.yaml` を初期化します。以降のフェーズは CLI command を通じて残りの directory を埋めます。

## Common Helper Commands

```sh
# 前提条件を検出
ghidra-agent-cli ghidra discover

# ターゲットワークスペースを作成
ghidra-agent-cli workspace init --target libfoo --binary ./libfoo.so
ghidra-agent-cli --target libfoo scope set --mode full

# baseline 証拠を出力
ghidra-agent-cli --target libfoo ghidra import
ghidra-agent-cli --target libfoo ghidra auto-analyze
ghidra-agent-cli --target libfoo ghidra export-baseline
ghidra-agent-cli --target libfoo functions list

# runtime と hotpath evidence
ghidra-agent-cli --target libfoo runtime record --key entrypoint --value 0x401000
ghidra-agent-cli --target libfoo hotpath add --addr 0x401000 --reason runtime

# metadata enrichment と Ghidra apply
ghidra-agent-cli --target libfoo metadata enrich-function \
  --addr 0x401000 \
  --name main \
  --prototype 'int(void)'
ghidra-agent-cli --target libfoo ghidra apply-renames
ghidra-agent-cli --target libfoo ghidra verify-renames
ghidra-agent-cli --target libfoo ghidra apply-signatures
ghidra-agent-cli --target libfoo ghidra verify-signatures

# selected decompilation と substitution record
ghidra-agent-cli --target libfoo ghidra decompile --fn-id fn_001 --addr 0x401000
ghidra-agent-cli --target libfoo substitute add \
  --fn-id fn_001 \
  --addr 0x401000 \
  --replacement 'return 0;'

# gate check
ghidra-agent-cli --target libfoo gate check --phase P1
ghidra-agent-cli --target libfoo gate list
```
