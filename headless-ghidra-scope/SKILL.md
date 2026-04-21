---
name: "headless-ghidra-scope"
description: "P0.5 sub-skill: define analysis scope by selecting functions, address ranges, or symbols from the initialized target."
phase: "P0.5"
---

# Headless Ghidra Scope — P0.5

P0.5 refines the analysis surface for a target. After P0 creates the workspace
and initial (possibly empty) `scope.yaml`, this phase populates scope entries
that guide downstream baseline extraction and decompilation.

## Required ghidra-agent-cli Commands

- `ghidra-agent-cli scope show`
- `ghidra-agent-cli scope set`
- `ghidra-agent-cli scope add-entry`
- `ghidra-agent-cli scope remove-entry`
- `ghidra-agent-cli gate check --phase P0.5`

## Inputs

- `artifacts/<target-id>/pipeline-state.yaml` (from P0)
- `artifacts/<target-id>/scope.yaml` (initially empty from P0)
- User guidance on what to analyze (function names, address ranges, symbols)
- Binary inspection results from `ghidra-agent-cli inspect binary`

## Outputs

- Updated `artifacts/<target-id>/scope.yaml` with non-empty `entries`
- Scope mode: `manual`, `auto`, or `mixed`

## Exit Expectations

- `scope.yaml` has a non-empty `entries` sequence.
- Each entry references a valid address, function name, or symbol.
- P0.5 gate passes (`scope.yaml exists with non-empty entries`).

## Constraints

- Do not run Ghidra analysis or baseline export in this phase.
- Do not modify `pipeline-state.yaml`.
- Only use `ghidra-agent-cli scope` commands for scope mutations.

## Next Step

- P0.5 gate passes → `headless-ghidra-baseline`
