# ghidra-agent-cli

`ghidra-agent-cli` is the bundled helper used by the Headless Ghidra skill
family. The skills invoke it to create target workspaces, inspect binaries, run
supported Ghidra and Frida operations, manage YAML artifacts, and check phase
gates.

Translations: [简体中文](./README.zh-CN.md) | [日本語](./README.ja-JP.md)

For normal use, start from the skill family README and ask your agent to use the
`headless-ghidra` skill. This file is an agent tool reference for command
semantics and troubleshooting. End users do not install, build, or run this CLI
manually during a normal skill workflow.

## Helper Runtime Prerequisites

These apply when the installed skill invokes the helper.

- Node.js >= 18 for the npm wrapper.
- A local Ghidra installation discoverable by `ghidra-agent-cli ghidra discover`
  or configured through `GHIDRA_INSTALL_DIR`.
- Optional Frida installation for `frida *` commands.

## Availability

The CLI ships with the skill family and is used from the installed skill
directory. If an agent reports that `ghidra-agent-cli` is unavailable, reinstall
or refresh the whole skill family so `headless-ghidra`, the phase skills, and
`ghidra-agent-cli` remain installed as sibling directories.

## Invocation

```sh
ghidra-agent-cli [GLOBAL FLAGS] <COMMAND> [COMMAND FLAGS]
```

Global flags:

- `--format yaml|json|toml` - Output format. Default: `yaml`.
- `--target <id>` - Target selector.
- `--workspace <path>` - Workspace root path.
- `--config-dir <PATH>` - Override config directory.
- `--data-dir <PATH>` - Override data directory.
- `--state-dir <PATH>` - Override state directory.
- `--cache-dir <PATH>` - Override cache directory.
- `--log-dir <PATH>` - Override log directory.
- `--lock-timeout <SECS>` - Lock acquisition timeout. Default: `30`.
- `--no-wait` - Do not wait for a workspace lock.
- `--help` - Show help.

Most target-specific commands require `--target <id>` or an active context set
with `context use`.

## Output And Errors

Successful commands emit a structured envelope in the selected format:

```yaml
status: ok
message: "<summary>"
data: <structured payload>
```

Errors are structured:

```json
{
  "code": "E_ERROR",
  "message": "description of what went wrong",
  "source": "main",
  "format": "json"
}
```

Known error codes include `E_ERROR`, `E_GATE_FAILED`, and `E_LOCK_TIMEOUT`.

## Command Groups

| Group                                                                           | Purpose                                                                                                       |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `workspace`                                                                     | Initialize a target workspace and manage phase state.                                                         |
| `scope`                                                                         | Manage `scope.yaml`.                                                                                          |
| `functions`, `callgraph`, `types`, `vtables`, `constants`, `strings`, `imports` | Read and curate baseline metadata.                                                                            |
| `third-party`                                                                   | Record third-party libraries, explicit no-third-party review, pristine sources, and function classifications. |
| `runtime`                                                                       | Manage `runtime/run-manifest.yaml` and `runtime/run-records/*.yaml`.                                          |
| `hotpath`                                                                       | Manage `runtime/hotpaths/call-chain.yaml`.                                                                    |
| `metadata`                                                                      | Manage P3 metadata such as renames and signatures.                                                            |
| `substitute`                                                                    | Manage P4 substitution records.                                                                               |
| `git-check`                                                                     | Check whether required artifacts are ready for review when gates ask for it.                                  |
| `execution-log`                                                                 | Append and inspect execution records.                                                                         |
| `progress`                                                                      | Helpers for older progress YAML.                                                                              |
| `gate`                                                                          | Run aggregate gate checks and inspect gate reports.                                                           |
| `ghidra`                                                                        | Discover Ghidra, import/analyze, export baseline, apply metadata, decompile, and rebuild.                     |
| `frida`                                                                         | Device, capture, compare, trace, run, and invoke helpers.                                                     |
| `inspect`                                                                       | Read-only binary inspection helpers.                                                                          |
| `context`                                                                       | Active target context helpers.                                                                                |
| `paths`                                                                         | Show resolved workspace and runtime paths.                                                                    |
| `validate`, `help`                                                              | Validation and help surfaces.                                                                                 |

## Workspace Layout

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

`workspace init` creates the base workspace structure and initializes
`pipeline-state.yaml` plus `scope.yaml`. Later phases populate the remaining
directories through CLI commands.

## Common Helper Commands

```sh
# Discover prerequisites
ghidra-agent-cli ghidra discover

# Create a target workspace
ghidra-agent-cli workspace init --target libfoo --binary ./libfoo.so
ghidra-agent-cli --target libfoo scope set --mode full

# Export baseline evidence
ghidra-agent-cli --target libfoo ghidra import
ghidra-agent-cli --target libfoo ghidra auto-analyze
ghidra-agent-cli --target libfoo ghidra export-baseline
ghidra-agent-cli --target libfoo functions list

# Runtime and hotpath evidence
ghidra-agent-cli --target libfoo runtime record --key entrypoint --value 0x401000
ghidra-agent-cli --target libfoo hotpath add --addr 0x401000 --reason runtime

# Metadata enrichment and Ghidra apply
ghidra-agent-cli --target libfoo metadata enrich-function \
  --addr 0x401000 \
  --name main \
  --prototype 'int(void)'
ghidra-agent-cli --target libfoo ghidra apply-renames
ghidra-agent-cli --target libfoo ghidra verify-renames
ghidra-agent-cli --target libfoo ghidra apply-signatures
ghidra-agent-cli --target libfoo ghidra verify-signatures

# Selected decompilation and substitution records
ghidra-agent-cli --target libfoo ghidra decompile --fn-id fn_001 --addr 0x401000
ghidra-agent-cli --target libfoo substitute add \
  --fn-id fn_001 \
  --addr 0x401000 \
  --replacement 'return 0;'

# Gate checks
ghidra-agent-cli --target libfoo gate check --phase P1
ghidra-agent-cli --target libfoo gate list
```
