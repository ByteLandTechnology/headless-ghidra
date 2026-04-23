---
name: "ghidra-agent-cli"
description: "Rust CLI reference for the headless-ghidra pipeline. Covers command syntax, flags, output contract, artifact paths, and workspace layout for all ghidra-agent-cli subcommands. Load when: constructing a ghidra-agent-cli command, interpreting its output, resolving a flag or artifact path question, or debugging CLI behavior. Do not load for P0–P6 workflow sequencing — use the headless-ghidra skill family instead."
---

# ghidra-agent-cli

## Description

`ghidra-agent-cli` is the shared Rust CLI for this repository. It owns the
command tree, workspace layout, YAML artifact semantics, output envelope, and
runtime behavior for supported Ghidra, Frida, progress, and gate operations.

This skill documents **how to use the CLI**. It does **not** define the P0–P6
workflow order, stage routing, or orchestration policy. Those are defined by
`headless-ghidra/SKILL.md` and the per-phase skills.

## Prerequisites

- Node.js >= 18
- Local Ghidra installation discoverable by `ghidra-agent-cli ghidra discover`
- Optional Frida installation for `frida *` commands

## Installation

<!-- SEMANTIC_RELEASE_VERSION -->
Install the matching CLI version into the skill directory (not global):

```sh
cd <skill-directory>
npm install ghidra-agent-cli@1.4.0
```

This creates `node_modules/.bin/ghidra-agent-cli` — a Node.js wrapper that
resolves the correct platform-specific binary via optional dependencies
(`@cli-forge-bin/ghidra-agent-cli-<os>-<cpu>`). Invoke via the full path:

```sh
./node_modules/.bin/ghidra-agent-cli [COMMAND] [FLAGS]
```

The version in the `npm install` command is updated automatically by
semantic-release during publish.

## Invocation

```sh
ghidra-agent-cli [GLOBAL FLAGS] <COMMAND> [COMMAND FLAGS]

# Development
cargo run -- [GLOBAL FLAGS] <COMMAND> [COMMAND FLAGS]
```

Global flags:

- `--format yaml|json|toml`
- `--target <id>`
- `--workspace <path>`
- `--config-dir`
- `--data-dir`
- `--state-dir`
- `--cache-dir`
- `--log-dir`
- `--lock-timeout`
- `--no-wait`
- `--help`

## Command Groups

| Group | Purpose |
|---|---|
| `workspace` | Initialize a target workspace and manage phase state |
| `scope` | Manage `scope.yaml` |
| `functions` / `callgraph` / `types` / `vtables` / `constants` / `strings` / `imports` | Manage baseline YAML metadata |
| `third-party` | Manage `third-party/identified.yaml` and vendored source metadata |
| `execution-log` | Append and inspect execution records |
| `progress` | Manage `decompilation/progress.yaml` and `decompilation/next-batch.yaml` |
| `gate` | Run aggregate gate checks and inspect gate reports |
| `ghidra` | Discover Ghidra, import/analyze, export baseline, apply changes, decompile, rebuild |
| `frida` | Device, capture, compare, trace, run, and invoke helpers |
| `inspect` | Read-only binary inspection helpers |
| `context` | Active context helpers |
| `paths` | Show resolved workspace/runtime paths |
| `validate` / `help` | Validation and help surfaces |

## Workspace Model

`ghidra-agent-cli` uses this repository-local layout:

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
├── third-party/
│   ├── identified.yaml
│   └── sources/
├── evidence-candidates.yaml
├── target-selection.yaml
├── decompilation/
│   ├── progress.yaml
│   ├── next-batch.yaml
│   └── functions/<fn_id>/
│       ├── decompilation-record.yaml
│       └── verification-result.yaml
├── gates/
└── scripts/
```

`workspace init` creates the base workspace structure and initializes
`pipeline-state.yaml` plus `scope.yaml`. Later phases populate the remaining
artifacts.

## Artifact Semantics

| Artifact | Meaning |
|---|---|
| `pipeline-state.yaml` | Current target, current phase, recorded binary path |
| `scope.yaml` | Explicit scope mode and entry list |
| `baseline/*.yaml` | Baseline metadata exported from Ghidra or curated through CLI commands |
| `third-party/identified.yaml` | Identified libraries plus function classifications |
| `evidence-candidates.yaml` | P2 evidence review output consumed by discovery |
| `target-selection.yaml` | P3 selected target/candidate set |
| `decompilation/progress.yaml` | Decompilation state per function |
| `decompilation/next-batch.yaml` | Current computed batch |
| `decompilation/functions/<fn_id>/decompilation-record.yaml` | Per-function decompilation provenance and metadata |
| `decompilation/functions/<fn_id>/verification-result.yaml` | Per-function runtime verification result |
| `gates/*-report.yaml` | Persisted gate check reports |

## Output Contract

Commands emit a structured envelope in the selected format:

```yaml
status: ok
message: "<summary>"
data: <structured payload>
```

Errors are structured and stable:

```json
{
  "code": "E_ERROR",
  "message": "description of what went wrong",
  "source": "main",
  "format": "json"
}
```

Known codes:

- `E_ERROR`
- `E_GATE_FAILED`
- `E_LOCK_TIMEOUT`

## Runtime Behavior

- Targeted commands normally require `--target <id>` unless an active context is
  already set.
- Supported mutating Ghidra and Frida operations run under the CLI lock model.
- `gate check` writes reports under `artifacts/<target-id>/gates/`.
- `help` and `--help` describe the CLI surface only; they are not the workflow
  specification.

## Examples

```sh
# Initialize a target
ghidra-agent-cli workspace init --target libfoo --binary ./libfoo.so

# Set scope and inspect state
ghidra-agent-cli --target libfoo scope set --mode full --entries 0x1000,0x2000
ghidra-agent-cli --target libfoo workspace state show

# Export and inspect baseline YAML
ghidra-agent-cli --target libfoo ghidra import
ghidra-agent-cli --target libfoo ghidra auto-analyze
ghidra-agent-cli --target libfoo ghidra export-baseline
ghidra-agent-cli --target libfoo functions list

# Progress and gate checks
ghidra-agent-cli --target libfoo progress compute-next-batch --max 8
ghidra-agent-cli --target libfoo ghidra decompile --batch
ghidra-agent-cli --target libfoo gate check --phase P1

# Frida helpers
ghidra-agent-cli frida device-list
ghidra-agent-cli frida trace --target ./bin/app --functions open,read
```

## Boundary

- Use this skill to answer: command names, flags, output shape, artifact paths,
  file semantics, and CLI behavior.
- Do not use this skill as the source of truth for P0–P6 sequencing, stage
  ownership, or workflow decisions. Those live in the `headless-ghidra` skill
  family.
