---
name: "headless-ghidra-intake"
description: "P0 sub-skill: confirm target identity, initialize the CLI workspace, and verify Ghidra availability."
phase: "P0"
---

# Headless Ghidra Intake — P0

P0 prepares a target for the rest of the workflow. It is responsible for
turning an input binary or archive path into a valid `ghidra-agent-cli`
workspace target.

## Required ghidra-agent-cli Commands

- `ghidra-agent-cli workspace init`
- `ghidra-agent-cli ghidra discover`
- `ghidra-agent-cli inspect binary`
- `ghidra-agent-cli gate check --phase P0`

Archive normalization and any repository-specific bootstrap scripts may still
run as backend details until the CLI exposes them directly.

## Inputs

- User-provided binary or archive path
- Workspace root
- Local environment needed for Ghidra discovery

## Outputs

- `targets/<target-id>/ghidra-projects/`
- `artifacts/<target-id>/pipeline-state.yaml`
- `artifacts/<target-id>/scope.yaml`
- Optional phase-owned intake records under `artifacts/<target-id>/intake/`

## Exit Expectations

- Target workspace exists and is addressable through `--target <id>`.
- `pipeline-state.yaml` records the selected binary path.
- `scope.yaml` exists, even if it still contains an empty entry list.
- Ghidra discovery and binary inspection have been run and reviewed.

## Constraints

- Do not run actual Ghidra analysis in this phase.
- Do not write baseline, evidence, decompilation, or verification outputs.
- Do not bypass `ghidra-agent-cli workspace init`, `ghidra discover`, or
  `inspect binary` for supported steps.

## Next Step

- P0 gate passes → `headless-ghidra-baseline`
