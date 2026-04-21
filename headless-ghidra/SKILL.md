---
name: "headless-ghidra"
description: "Global orchestrator for the headless Ghidra YAML-first decompilation pipeline. Reads artifacts/<target>/pipeline-state.yaml, dispatches P0–P6 phase skills, executes programmatic gate checks, and manages user interaction. Performs zero analysis work itself."
---

# Headless Ghidra — Global Orchestrator

This skill is the workflow authority for the repository. It defines the P0–P6
sequence, dispatch rules, and artifact hand-off points. `ghidra-agent-cli`
remains the tool authority for command syntax and YAML artifact semantics.

## Required Shared Tool Contract

- `ghidra-agent-cli` is the mandatory shared interface for supported workspace,
  metadata, Ghidra, Frida, progress, validation, and gate operations.
- Phase skills must name the exact `ghidra-agent-cli` subcommands they use.
- Lower-level shell scripts and Java helpers are backend details. They must not
  replace the CLI as the primary interface when the CLI already supports the
  action.

## Pipeline

```text
P0 Intake → P0.5 Scope → P1 Baseline → P2 Evidence → [P3 Discovery → P4+P5 Decompile → P6 Verify]*
```

| Phase | Skill | Purpose | Primary outputs |
|---|---|---|---|
| P0 | [`headless-ghidra-intake`](../headless-ghidra-intake/SKILL.md) | Initialize target workspace and discover runtime prerequisites | `pipeline-state.yaml`, `scope.yaml`, `targets/<id>/ghidra-projects/` |
| P0.5 | [`headless-ghidra-scope`](../headless-ghidra-scope/SKILL.md) | Define analysis scope (functions, addresses, symbols) | Updated `scope.yaml` with non-empty entries |
| P1 | [`headless-ghidra-baseline`](../headless-ghidra-baseline/SKILL.md) | Export baseline YAML metadata from Ghidra | `baseline/*.yaml` |
| P2 | [`headless-ghidra-evidence`](../headless-ghidra-evidence/SKILL.md) | Review baseline evidence and third-party signals | `evidence-candidates.yaml`, `third-party/identified.yaml` |
| P3 | [`headless-ghidra-discovery`](../headless-ghidra-discovery/SKILL.md) | Select the next frontier batch | `target-selection.yaml`, refreshed `next-batch.yaml` |
| P4+P5 | [`headless-ghidra-batch-decompile`](../headless-ghidra-batch-decompile/SKILL.md) | Apply metadata, decompile, and record per-function outputs | `decompilation/functions/<fn_id>/decompilation-record.yaml` |
| P6 | [`headless-ghidra-frida-verify`](../headless-ghidra-frida-verify/SKILL.md) | Record and compare runtime behavior | `decompilation/functions/<fn_id>/verification-result.yaml` |

## Shared Artifact Contract

All phases work inside this repository-local layout:

```text
targets/<target-id>/ghidra-projects/

artifacts/<target-id>/
├── pipeline-state.yaml
├── scope.yaml
├── intake/
├── baseline/
├── third-party/
├── evidence-candidates.yaml
├── target-selection.yaml
├── decompilation/
│   ├── progress.yaml
│   ├── next-batch.yaml
│   └── functions/<fn_id>/
└── gates/
```

The orchestrator treats `pipeline-state.yaml` as the current target-level state
record and relies on the phase-owned YAML artifacts above for hand-offs.

## Orchestrator Responsibilities

1. Detect or resume the active target.
2. Read `artifacts/<target-id>/pipeline-state.yaml`.
3. Dispatch the correct phase skill for the current stage.
4. Run `ghidra-agent-cli gate check --phase ...` at each transition.
6. Advance phase state only after the gate passes.
7. Handle user dialogs such as resume/restart, optional Frida supplementation,
   batch confirmation, divergence review, and completion.

## Gate Policy

- P0–P4 are normally target-level transitions.
- P5 and P6 are evaluated per function in the workflow, even when the CLI still
  exposes aggregate target-level checks for some paths.
- `ghidra-agent-cli gate check` is the authoritative gate validation for all
  pipeline phases (P0–P6). The legacy `gate-check.sh` has been removed.

## Required ghidra-agent-cli Commands

- `ghidra-agent-cli context use`
- `ghidra-agent-cli context show`
- `ghidra-agent-cli context clear`
- `ghidra-agent-cli workspace state show`
- `ghidra-agent-cli workspace state set-phase`
- `ghidra-agent-cli gate check`
- `ghidra-agent-cli validate`
- `ghidra-agent-cli progress compute-next-batch`
- `ghidra-agent-cli progress show`

## Strict Prohibitions

- ⛔ Must not execute analysis work itself.
- ⛔ Must not edit baseline, evidence, decompilation, or verification artifacts
  directly except for explicit state updates it owns.
- ⛔ Must not bypass `ghidra-agent-cli` for supported state, progress, context,
  validation, or gate operations.
- ⛔ Must not accept alternate decompilation backends in place of Ghidra.

## Next Skill Routing

- P0 complete → `headless-ghidra-scope`
- P0.5 complete → `headless-ghidra-baseline`
- P1 complete → `headless-ghidra-evidence`
- P2 complete → `headless-ghidra-discovery`
- P3 complete → `headless-ghidra-batch-decompile`
- P5 complete for all selected functions → `headless-ghidra-frida-verify`
- P6 complete for all selected functions → either loop back to P3 or finish
