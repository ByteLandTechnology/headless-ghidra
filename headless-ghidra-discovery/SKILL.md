---
name: "headless-ghidra-discovery"
description: "P3 sub-skill: analyze verified boundaries, evidence, and progress to record the next target selection in YAML."
phase: "P3"
---

# Headless Ghidra Discovery — P3

P3 turns the reviewed evidence and current progress state into the next selected
frontier target or batch.

## Required ghidra-agent-cli Commands

- `ghidra-agent-cli workspace state show`
- `ghidra-agent-cli functions list`
- `ghidra-agent-cli callgraph callers`
- `ghidra-agent-cli callgraph callees`
- `ghidra-agent-cli progress compute-next-batch`
- `ghidra-agent-cli progress show`
- `ghidra-agent-cli progress list`
- `ghidra-agent-cli gate check --phase P3`

## Inputs

- `artifacts/<target-id>/pipeline-state.yaml`
- `artifacts/<target-id>/evidence-candidates.yaml`
- `artifacts/<target-id>/baseline/functions.yaml`
- `artifacts/<target-id>/baseline/callgraph.yaml`
- `artifacts/<target-id>/decompilation/progress.yaml`

## Outputs

- `artifacts/<target-id>/target-selection.yaml`
- Refreshed `artifacts/<target-id>/decompilation/next-batch.yaml`

## Exit Expectations

- `target-selection.yaml` records the chosen default target or candidate set.
- `next-batch.yaml` reflects the currently selected worklist for downstream
  decompilation.
- The selection is reproducible from the recorded evidence and progress inputs.

## Constraints

- Do not decompile anything in this phase.
- Do not rewrite historical per-function outputs.
- Do not bypass `ghidra-agent-cli` for supported state, baseline, callgraph,
  progress, or gate operations.

## Next Step

- P3 gate passes → `headless-ghidra-batch-decompile`
