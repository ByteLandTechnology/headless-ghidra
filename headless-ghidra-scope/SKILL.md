---
name: "headless-ghidra-scope"
description: "Deprecated compatibility-only P0.5 alias: scope selection now belongs to P0 Intake."
phase: "P0.5"
---

# Headless Ghidra Scope — P0.5

Deprecated compatibility-only alias. Scope selection is now part of P0 Intake;
do not route new workflows here as a primary pipeline stage. Keep this document
only for older CLI phase aliases or historical runs that still mention P0.5.

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
- Compatibility P0.5 gate passes when older runs still require it.

## Constraints

- Do not run Ghidra analysis or baseline export in this phase.
- Do not modify `pipeline-state.yaml`.
- Only use `ghidra-agent-cli scope` commands for scope mutations.

## Next Step

- Compatibility P0.5 gate passes → `headless-ghidra-baseline`
