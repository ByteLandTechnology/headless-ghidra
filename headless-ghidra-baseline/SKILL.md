---
name: "headless-ghidra-baseline"
description: "P1 sub-skill: run Ghidra import/analysis and export baseline YAML metadata without decompiling function bodies."
phase: "P1"
---

# Headless Ghidra Baseline — P1

P1 runs the initial Ghidra import and auto-analysis, then exports the baseline
YAML metadata that the later phases consume.

## Required ghidra-agent-cli Commands

- `ghidra-agent-cli ghidra import`
- `ghidra-agent-cli ghidra auto-analyze`
- `ghidra-agent-cli ghidra export-baseline`
- `ghidra-agent-cli gate check --phase P1`

The shell wrappers and Java Ghidra scripts remain the backend implementation for
these commands. The workflow contract is the CLI surface plus the YAML outputs
below.

## Inputs

- `artifacts/<target-id>/pipeline-state.yaml`
- `targets/<target-id>/ghidra-projects/`

## Outputs

- `artifacts/<target-id>/baseline/functions.yaml`
- `artifacts/<target-id>/baseline/callgraph.yaml`
- `artifacts/<target-id>/baseline/types.yaml`
- `artifacts/<target-id>/baseline/vtables.yaml`
- `artifacts/<target-id>/baseline/constants.yaml`
- `artifacts/<target-id>/baseline/strings.yaml`
- `artifacts/<target-id>/baseline/imports.yaml`

## Exit Expectations

- All required baseline YAML files exist and are readable.
- The baseline set is sufficient for evidence review and target selection.
- No P4/P5 decompilation artifacts are created in this phase.

## Constraints

- Do not decompile function bodies in P1.
- Do not apply renames or signatures in P1.
- Do not modify `pipeline-state.yaml` except through sanctioned state changes.
- Do not bypass `ghidra-agent-cli` for import, analysis, export, or supported
  gate checks.

## Next Step

- P1 gate passes → `headless-ghidra-evidence`
